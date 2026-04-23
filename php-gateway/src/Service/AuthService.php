<?php

declare(strict_types=1);

namespace ArchivioParlante\Service;

use ArchivioParlante\Exception\AuthenticationException;
use ArchivioParlante\Exception\ValidationException;
use ArchivioParlante\Repository\UserRepository;
use Psr\Log\LoggerInterface;

/**
 * Authentication business logic service
 *
 * Orchestrates:
 * - User registration with password validation and hashing
 * - Login with credential verification
 * - Token refresh
 * - Logout with token revocation
 * - Current user retrieval
 *
 * Security:
 * - Bcrypt password hashing (cost factor 12)
 * - Timing-safe password verification
 * - Password strength validation (min 8 chars, mixed case, digit)
 * - Never logs passwords or sensitive data
 */
final class AuthService
{
    private const PASSWORD_MIN_LENGTH = 8;
    private const BCRYPT_COST = 12;

    public function __construct(
        private UserRepository $userRepository,
        private JwtService $jwtService,
        private RedisSessionManager $sessionManager,
        private AuditLogger $auditLogger,
        private LoggerInterface $logger
    ) {
    }

    /**
     * Register new user
     *
     * @param string $email User email
     * @param string $password Plain password
     * @param string $fullName User full name
     * @param string $ipAddress Client IP address (for audit logging)
     * @param string $userAgent Client User-Agent (for audit logging)
     * @return array{access_token: string, refresh_token: string, user: array{id: int, email: string, full_name: string, role: string}} Tokens and user data
     * @throws ValidationException If validation fails
     * @throws \RuntimeException If database operation fails
     */
    public function register(string $email, string $password, string $fullName, string $ipAddress, string $userAgent): array
    {
        // Validate email format
        if (!filter_var($email, FILTER_VALIDATE_EMAIL)) {
            throw ValidationException::invalidEmail();
        }

        // Validate password strength
        $passwordErrors = $this->validatePasswordStrength($password);
        if (!empty($passwordErrors)) {
            throw ValidationException::weakPassword($passwordErrors);
        }

        // Check if email already exists
        if ($this->userRepository->existsByEmail($email)) {
            throw ValidationException::duplicateEmail();
        }

        // Hash password with bcrypt (bcrypt never fails with valid params)
        $passwordHash = password_hash($password, PASSWORD_BCRYPT, ['cost' => self::BCRYPT_COST]);

        // Create user
        $userId = $this->userRepository->create($email, $passwordHash, $fullName);

        // Generate tokens
        $accessToken = $this->jwtService->generateAccessToken($userId, $email, 'user');
        $refreshToken = $this->jwtService->generateRefreshToken();

        // Store refresh token in Redis
        $this->sessionManager->storeRefreshToken($refreshToken, $userId);
        $this->sessionManager->updateSessionActivity($userId);

        // Log successful registration to audit log
        $this->auditLogger->logAuthEvent(
            eventType: 'register_success',
            userId: $userId,
            ipAddress: $ipAddress,
            userAgent: $userAgent,
            requestPath: '/api/auth/register',
            requestMethod: 'POST',
            statusCode: 201
        );

        $this->logger->info('User registered successfully', [
            'user_id' => $userId,
            'email' => $email,
        ]);

        return [
            'access_token' => $accessToken,
            'refresh_token' => $refreshToken,
            'user' => [
                'id' => $userId,
                'email' => $email,
                'full_name' => $fullName,
                'role' => 'user',
            ],
        ];
    }

    /**
     * Login user with email and password
     *
     * @param string $email User email
     * @param string $password Plain password
     * @param string $ipAddress Client IP address (for rate limiting and audit logging)
     * @param string $userAgent Client User-Agent (for audit logging)
     * @return array{access_token: string, refresh_token: string, user: array{id: int, email: string, full_name: string, role: string, last_login_at: string|null}} Tokens and user data
     * @throws AuthenticationException If credentials are invalid
     * @throws \RuntimeException If database operation fails
     */
    public function login(string $email, string $password, string $ipAddress, string $userAgent): array
    {
        // Find user by email
        $user = $this->userRepository->findByEmail($email);

        if ($user === null) {
            // Log failed attempt
            $this->logger->warning('Login attempt with non-existent email', [
                'email' => $email,
                'ip' => $ipAddress,
            ]);

            $this->sessionManager->incrementLoginAttempts($ipAddress);

            // Log failed login to audit log
            $this->auditLogger->logAuthEvent(
                eventType: 'login_failed',
                userId: null,
                ipAddress: $ipAddress,
                userAgent: $userAgent,
                requestPath: '/api/auth/login',
                requestMethod: 'POST',
                statusCode: 401,
                eventData: ['reason' => 'user_not_found']
            );

            // Generic error message to prevent user enumeration
            throw AuthenticationException::invalidCredentials();
        }

        // Verify password (timing-safe comparison)
        if (!password_verify($password, $user['password_hash'])) {
            $this->logger->warning('Login attempt with wrong password', [
                'user_id' => $user['id'],
                'email' => $email,
                'ip' => $ipAddress,
            ]);

            $this->sessionManager->incrementLoginAttempts($ipAddress);

            // Log failed login to audit log
            $this->auditLogger->logAuthEvent(
                eventType: 'login_failed',
                userId: (int) $user['id'],
                ipAddress: $ipAddress,
                userAgent: $userAgent,
                requestPath: '/api/auth/login',
                requestMethod: 'POST',
                statusCode: 401,
                eventData: ['reason' => 'invalid_password']
            );

            throw AuthenticationException::invalidCredentials();
        }

        // Check if user is active
        if (!$user['is_active']) {
            $this->logger->warning('Login attempt for inactive user', [
                'user_id' => $user['id'],
                'email' => $email,
            ]);

            throw new AuthenticationException('User account is disabled', 403);
        }

        // Update last login timestamp
        $this->userRepository->updateLastLogin((int) $user['id']);

        // Generate tokens
        $accessToken = $this->jwtService->generateAccessToken(
            (int) $user['id'],
            $user['email'],
            $user['role']
        );
        $refreshToken = $this->jwtService->generateRefreshToken();

        // Store refresh token in Redis
        $this->sessionManager->storeRefreshToken($refreshToken, (int) $user['id']);
        $this->sessionManager->updateSessionActivity((int) $user['id']);

        // Reset failed login attempts on successful login
        $this->sessionManager->resetLoginAttempts($ipAddress);

        // Log successful login to audit log
        $this->auditLogger->logAuthEvent(
            eventType: 'login_success',
            userId: (int) $user['id'],
            ipAddress: $ipAddress,
            userAgent: $userAgent,
            requestPath: '/api/auth/login',
            requestMethod: 'POST',
            statusCode: 200
        );

        $this->logger->info('User logged in successfully', [
            'user_id' => $user['id'],
            'email' => $email,
            'ip' => $ipAddress,
        ]);

        return [
            'access_token' => $accessToken,
            'refresh_token' => $refreshToken,
            'user' => [
                'id' => (int) $user['id'],
                'email' => $user['email'],
                'full_name' => $user['full_name'],
                'role' => $user['role'],
                'last_login_at' => $user['last_login_at'],
            ],
        ];
    }

    /**
     * Refresh access token using refresh token
     *
     * @param string $refreshToken Refresh token
     * @param string $ipAddress Client IP address (for audit logging)
     * @param string $userAgent Client User-Agent (for audit logging)
     * @return array{access_token: string} New access token
     * @throws AuthenticationException If refresh token is invalid or expired
     * @throws \RuntimeException If database operation fails
     */
    public function refresh(string $refreshToken, string $ipAddress, string $userAgent): array
    {
        // Validate refresh token in Redis
        $userId = $this->sessionManager->validateRefreshToken($refreshToken);

        if ($userId === null) {
            $this->logger->warning('Refresh attempt with invalid token');
            throw AuthenticationException::invalidRefreshToken();
        }

        // Fetch user data
        $user = $this->userRepository->findById($userId);

        if ($user === null || !$user['is_active']) {
            $this->logger->warning('Refresh attempt for non-existent or inactive user', [
                'user_id' => $userId,
            ]);

            throw AuthenticationException::invalidRefreshToken();
        }

        // Generate new access token (NOT new refresh token)
        $accessToken = $this->jwtService->generateAccessToken(
            (int) $user['id'],
            $user['email'],
            $user['role']
        );

        $this->sessionManager->updateSessionActivity((int) $user['id']);

        // Log token refresh to audit log
        $this->auditLogger->logAuthEvent(
            eventType: 'token_refresh',
            userId: (int) $user['id'],
            ipAddress: $ipAddress,
            userAgent: $userAgent,
            requestPath: '/api/auth/refresh',
            requestMethod: 'POST',
            statusCode: 200
        );

        $this->logger->info('Access token refreshed', [
            'user_id' => $user['id'],
        ]);

        return [
            'access_token' => $accessToken,
        ];
    }

    /**
     * Logout user by revoking refresh token
     *
     * @param string $refreshToken Refresh token to revoke
     * @return void
     */
    public function logout(string $refreshToken): void
    {
        // Revoke refresh token from Redis
        $this->sessionManager->revokeRefreshToken($refreshToken);

        $this->logger->info('User logged out');
    }

    /**
     * Get current user data by ID
     *
     * @param int $userId User ID
     * @return array<string, mixed> User data (without password_hash)
     * @throws AuthenticationException If user not found
     */
    public function getCurrentUser(int $userId): array
    {
        $user = $this->userRepository->findById($userId);

        if ($user === null) {
            throw new AuthenticationException('User not found', 404);
        }

        // Remove sensitive data
        unset($user['password_hash']);

        return $user;
    }

    /**
     * Validate password strength
     *
     * Requirements:
     * - Minimum 8 characters
     * - At least one uppercase letter
     * - At least one lowercase letter
     * - At least one digit
     *
     * @param string $password Plain password
     * @return string[] Array of error messages (empty if valid)
     */
    private function validatePasswordStrength(string $password): array
    {
        $errors = [];

        if (strlen($password) < self::PASSWORD_MIN_LENGTH) {
            $errors[] = sprintf('Password must be at least %d characters', self::PASSWORD_MIN_LENGTH);
        }

        if (!preg_match('/[A-Z]/', $password)) {
            $errors[] = 'Password must contain at least one uppercase letter';
        }

        if (!preg_match('/[a-z]/', $password)) {
            $errors[] = 'Password must contain at least one lowercase letter';
        }

        if (!preg_match('/\d/', $password)) {
            $errors[] = 'Password must contain at least one digit';
        }

        return $errors;
    }
}

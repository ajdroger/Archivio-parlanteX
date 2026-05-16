<?php

declare(strict_types=1);

namespace ArchivioParlante\Service;

use ArchivioParlante\Exception\AuthenticationException;
use Firebase\JWT\JWT;
use Firebase\JWT\Key;
use Firebase\JWT\ExpiredException;
use Firebase\JWT\SignatureInvalidException;
use Psr\Log\LoggerInterface;

/**
 * JWT token generation and validation service
 *
 * Implements dual-token strategy:
 * - Access token: Short-lived (15 min), JWT signed with HS256
 * - Refresh token: Long-lived (7 days), random 64-byte hex stored in Redis
 *
 * Security:
 * - HS256 algorithm with JWT_SECRET from environment
 * - Access token payload: user_id, email, role, iat, exp
 * - Refresh token: cryptographically secure random bytes
 */
class JwtService
{
    private const ACCESS_TOKEN_TTL = 900; // 15 minutes in seconds
    private const ALGORITHM = 'HS256';
    private const ISSUER = 'archivio-parlante-php-gateway';

    public function __construct(
        private string $jwtSecret,
        private LoggerInterface $logger
    ) {
        if (strlen($jwtSecret) < 32) {
            throw new \InvalidArgumentException('JWT_SECRET must be at least 32 characters');
        }
    }

    /**
     * Generate access token for authenticated user
     *
     * @param int $userId User ID
     * @param string $email User email
     * @param string $role User role (admin, user, viewer)
     * @return string JWT access token
     */
    public function generateAccessToken(int $userId, string $email, string $role): string
    {
        $issuedAt = time();
        $expiresAt = $issuedAt + self::ACCESS_TOKEN_TTL;

        $payload = [
            'iss' => self::ISSUER,
            'sub' => (string) $userId,
            'email' => $email,
            'role' => $role,
            'iat' => $issuedAt,
            'exp' => $expiresAt,
        ];

        $token = JWT::encode($payload, $this->jwtSecret, self::ALGORITHM);

        $this->logger->debug('Generated access token', [
            'user_id' => $userId,
            'email' => $email,
            'expires_at' => $expiresAt,
        ]);

        return $token;
    }

    /**
     * Generate refresh token (random 64-byte hex string)
     *
     * @return string Refresh token (128 characters hex)
     * @throws \RuntimeException If random_bytes fails
     */
    public function generateRefreshToken(): string
    {
        try {
            $randomBytes = random_bytes(64);
            $token = bin2hex($randomBytes);

            $this->logger->debug('Generated refresh token');

            return $token;
        } catch (\Exception $e) {
            $this->logger->error('Failed to generate refresh token', [
                'error' => $e->getMessage(),
            ]);

            throw new \RuntimeException('Failed to generate secure random token', 0, $e);
        }
    }

    /**
     * Validate and decode access token
     *
     * @param string $token JWT access token
     * @return array{sub: string, email: string, role: string, iat: int, exp: int} Decoded payload
     * @throws AuthenticationException If token is invalid, expired, or tampered
     */
    public function validateAccessToken(string $token): array
    {
        try {
            $decoded = JWT::decode($token, new Key($this->jwtSecret, self::ALGORITHM));

            // Convert stdClass to array
            $payload = [
                'sub' => $decoded->sub,
                'email' => $decoded->email,
                'role' => $decoded->role,
                'iat' => $decoded->iat,
                'exp' => $decoded->exp,
            ];

            $this->logger->debug('Access token validated', [
                'user_id' => $payload['sub'],
                'email' => $payload['email'],
            ]);

            return $payload;
        } catch (ExpiredException $e) {
            $this->logger->debug('Access token expired', [
                'error' => $e->getMessage(),
            ]);

            throw AuthenticationException::invalidToken('Access token expired');
        } catch (SignatureInvalidException $e) {
            $this->logger->warning('Invalid token signature detected', [
                'error' => $e->getMessage(),
            ]);

            throw AuthenticationException::invalidToken('Invalid token signature');
        } catch (\Exception $e) {
            $this->logger->warning('Token validation failed', [
                'error' => $e->getMessage(),
            ]);

            throw AuthenticationException::invalidToken('Malformed token');
        }
    }

    /**
     * Extract token from Authorization header
     *
     * @param string|null $authHeader Authorization header value (e.g., "Bearer eyJ...")
     * @return string|null JWT token or null if header is invalid
     */
    public function extractTokenFromHeader(?string $authHeader): ?string
    {
        if ($authHeader === null || $authHeader === '') {
            return null;
        }

        // Expected format: "Bearer <token>"
        if (!str_starts_with($authHeader, 'Bearer ')) {
            $this->logger->debug('Authorization header does not start with Bearer');
            return null;
        }

        $token = substr($authHeader, 7); // Remove "Bearer " prefix

        if ($token === '') {
            $this->logger->debug('Empty token after Bearer prefix');
            return null;
        }

        return $token;
    }

    /**
     * Decode token without validation (for debugging)
     *
     * WARNING: Do NOT use for authentication! Use validateAccessToken() instead.
     *
     * @param string $token JWT token
     * @return array<string, mixed>|null Decoded payload or null if malformed
     */
    public function decodeWithoutValidation(string $token): ?array
    {
        try {
            $parts = explode('.', $token);

            if (count($parts) !== 3) {
                return null;
            }

            $payload = json_decode(base64_decode($parts[1]), true);

            return is_array($payload) ? $payload : null;
        } catch (\Exception $e) {
            $this->logger->debug('Failed to decode token', [
                'error' => $e->getMessage(),
            ]);

            return null;
        }
    }
}

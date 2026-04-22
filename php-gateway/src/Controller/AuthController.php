<?php

declare(strict_types=1);

namespace ArchivioParlante\Controller;

use ArchivioParlante\Exception\AuthenticationException;
use ArchivioParlante\Exception\ValidationException;
use ArchivioParlante\Service\AuthService;
use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Log\LoggerInterface;

/**
 * Authentication controller
 *
 * Endpoints:
 * - POST /api/auth/register - Register new user
 * - POST /api/auth/login - Login with credentials
 * - POST /api/auth/refresh - Refresh access token
 * - POST /api/auth/logout - Logout and revoke tokens
 * - GET /api/auth/me - Get current user info
 */
final class AuthController
{
    public function __construct(
        private AuthService $authService,
        private LoggerInterface $logger
    ) {
    }

    /**
     * Register new user
     *
     * POST /api/auth/register
     * Body: {"email": "user@example.com", "password": "SecurePass123", "name": "Full Name"}
     * Response 201: {"access_token": "...", "refresh_token": "...", "user": {...}}
     * Response 400: {"error": "Validation failed", "errors": {"field": ["message"]}}
     * Response 500: {"error": "Internal server error"}
     */
    public function register(Request $request, Response $response): Response
    {
        try {
            $body = $this->parseJsonBody($request);

            // Validate required fields
            $this->validateRequiredFields($body, ['email', 'password', 'name']);

            $result = $this->authService->register(
                $body['email'],
                $body['password'],
                $body['name']
            );

            $response->getBody()->write(json_encode($result, JSON_THROW_ON_ERROR));

            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(201);
        } catch (ValidationException $e) {
            return $this->jsonErrorResponse($response, $e->getMessage(), 400, $e->getErrors());
        } catch (\Exception $e) {
            $this->logger->error('Registration failed', ['error' => $e->getMessage()]);
            return $this->jsonErrorResponse($response, 'Registration failed', 500);
        }
    }

    /**
     * Login with email and password
     *
     * POST /api/auth/login
     * Body: {"email": "user@example.com", "password": "SecurePass123"}
     * Response 200: {"access_token": "...", "refresh_token": "...", "user": {...}}
     * Response 400: {"error": "Validation failed"}
     * Response 401: {"error": "Invalid credentials"}
     * Response 429: {"error": "Too many requests", "retry_after": 900}
     * Response 500: {"error": "Internal server error"}
     */
    public function login(Request $request, Response $response): Response
    {
        try {
            $body = $this->parseJsonBody($request);

            // Validate required fields
            $this->validateRequiredFields($body, ['email', 'password']);

            // Get client IP address
            $serverParams = $request->getServerParams();
            $ipAddress = $serverParams['REMOTE_ADDR'] ?? '127.0.0.1';

            $result = $this->authService->login(
                $body['email'],
                $body['password'],
                $ipAddress
            );

            $response->getBody()->write(json_encode($result, JSON_THROW_ON_ERROR));

            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(200);
        } catch (AuthenticationException $e) {
            return $this->jsonErrorResponse($response, $e->getMessage(), $e->getCode());
        } catch (ValidationException $e) {
            return $this->jsonErrorResponse($response, $e->getMessage(), 400, $e->getErrors());
        } catch (\Exception $e) {
            $this->logger->error('Login failed', ['error' => $e->getMessage()]);
            return $this->jsonErrorResponse($response, 'Login failed', 500);
        }
    }

    /**
     * Refresh access token
     *
     * POST /api/auth/refresh
     * Body: {"refresh_token": "..."}
     * Response 200: {"access_token": "..."}
     * Response 400: {"error": "Validation failed"}
     * Response 401: {"error": "Invalid refresh token"}
     * Response 500: {"error": "Internal server error"}
     */
    public function refresh(Request $request, Response $response): Response
    {
        try {
            $body = $this->parseJsonBody($request);

            // Validate required fields
            $this->validateRequiredFields($body, ['refresh_token']);

            $result = $this->authService->refresh($body['refresh_token']);

            $response->getBody()->write(json_encode($result, JSON_THROW_ON_ERROR));

            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(200);
        } catch (AuthenticationException $e) {
            return $this->jsonErrorResponse($response, $e->getMessage(), $e->getCode());
        } catch (ValidationException $e) {
            return $this->jsonErrorResponse($response, $e->getMessage(), 400, $e->getErrors());
        } catch (\Exception $e) {
            $this->logger->error('Token refresh failed', ['error' => $e->getMessage()]);
            return $this->jsonErrorResponse($response, 'Token refresh failed', 500);
        }
    }

    /**
     * Logout user
     *
     * POST /api/auth/logout
     * Headers: Authorization: Bearer <access_token>
     * Body: {"refresh_token": "..."}
     * Response 204: (No Content)
     * Response 400: {"error": "Validation failed"}
     * Response 401: {"error": "Unauthorized"}
     * Response 500: {"error": "Internal server error"}
     */
    public function logout(Request $request, Response $response): Response
    {
        try {
            $body = $this->parseJsonBody($request);

            // Validate required fields
            $this->validateRequiredFields($body, ['refresh_token']);

            $this->authService->logout($body['refresh_token']);

            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(204);
        } catch (ValidationException $e) {
            return $this->jsonErrorResponse($response, $e->getMessage(), 400, $e->getErrors());
        } catch (\Exception $e) {
            $this->logger->error('Logout failed', ['error' => $e->getMessage()]);
            return $this->jsonErrorResponse($response, 'Logout failed', 500);
        }
    }

    /**
     * Get current user info
     *
     * GET /api/auth/me
     * Headers: Authorization: Bearer <access_token>
     * Response 200: {"id": 1, "email": "user@example.com", "full_name": "...", "role": "user", ...}
     * Response 401: {"error": "Unauthorized"}
     * Response 404: {"error": "User not found"}
     * Response 500: {"error": "Internal server error"}
     */
    public function me(Request $request, Response $response): Response
    {
        try {
            // User data is set by AuthMiddleware in request attributes
            $user = $request->getAttribute('user');

            if ($user === null) {
                throw new AuthenticationException('User not authenticated', 401);
            }

            // Fetch full user data from database
            $userId = (int) $user['sub'];
            $userData = $this->authService->getCurrentUser($userId);

            $response->getBody()->write(json_encode($userData, JSON_THROW_ON_ERROR));

            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(200);
        } catch (AuthenticationException $e) {
            return $this->jsonErrorResponse($response, $e->getMessage(), $e->getCode());
        } catch (\Exception $e) {
            $this->logger->error('Failed to get current user', ['error' => $e->getMessage()]);
            return $this->jsonErrorResponse($response, 'Failed to get user info', 500);
        }
    }

    /**
     * Parse JSON request body
     *
     * @param Request $request PSR-7 request
     * @return array<string, mixed> Parsed body
     * @throws ValidationException If JSON is invalid
     */
    private function parseJsonBody(Request $request): array
    {
        $body = (string) $request->getBody();

        if ($body === '') {
            throw ValidationException::withErrors(['body' => ['Request body is required']]);
        }

        try {
            $data = json_decode($body, true, 512, JSON_THROW_ON_ERROR);

            if (!is_array($data)) {
                throw new \InvalidArgumentException('Request body must be a JSON object');
            }

            return $data;
        } catch (\JsonException $e) {
            throw ValidationException::withErrors(['body' => ['Invalid JSON format']]);
        }
    }

    /**
     * Validate required fields in request body
     *
     * @param array<string, mixed> $data Request data
     * @param string[] $requiredFields Array of required field names
     * @return void
     * @throws ValidationException If any required field is missing
     */
    private function validateRequiredFields(array $data, array $requiredFields): void
    {
        $errors = [];

        foreach ($requiredFields as $field) {
            if (!isset($data[$field]) || $data[$field] === '' || $data[$field] === null) {
                $errors[$field] = ['This field is required'];
            }
        }

        if (!empty($errors)) {
            throw ValidationException::withErrors($errors);
        }
    }

    /**
     * Create JSON error response
     *
     * @param Response $response PSR-7 response
     * @param string $message Error message
     * @param int $statusCode HTTP status code
     * @param array<string, string[]>|null $errors Optional validation errors
     * @return Response Modified response
     */
    private function jsonErrorResponse(
        Response $response,
        string $message,
        int $statusCode,
        ?array $errors = null
    ): Response {
        $body = ['error' => $message];

        if ($errors !== null && !empty($errors)) {
            $body['errors'] = $errors;
        }

        $response->getBody()->write(json_encode($body, JSON_THROW_ON_ERROR));

        return $response
            ->withHeader('Content-Type', 'application/json')
            ->withStatus($statusCode);
    }
}

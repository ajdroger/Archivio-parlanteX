<?php

declare(strict_types=1);

namespace ArchivioParlante\Exception;

use RuntimeException;

/**
 * Exception thrown when authentication fails
 *
 * This exception is thrown when:
 * - Invalid credentials are provided during login
 * - JWT token is invalid, expired, or tampered
 * - Required authentication headers are missing
 * - Refresh token is invalid or revoked
 *
 * HTTP Status Code: 401 Unauthorized
 */
final class AuthenticationException extends RuntimeException
{
    /**
     * Create a new authentication exception
     *
     * @param string $message Error message (never include sensitive data like passwords)
     * @param int $code Error code (default: 401)
     * @param \Throwable|null $previous Previous exception for chaining
     */
    public function __construct(string $message = 'Authentication failed', int $code = 401, ?\Throwable $previous = null)
    {
        parent::__construct($message, $code, $previous);
    }

    /**
     * Create exception for invalid credentials
     */
    public static function invalidCredentials(): self
    {
        return new self('Invalid email or password', 401);
    }

    /**
     * Create exception for invalid or expired token
     */
    public static function invalidToken(string $reason = 'Invalid or expired token'): self
    {
        return new self($reason, 401);
    }

    /**
     * Create exception for missing authorization header
     */
    public static function missingAuthHeader(): self
    {
        return new self('Missing Authorization header', 401);
    }

    /**
     * Create exception for invalid refresh token
     */
    public static function invalidRefreshToken(): self
    {
        return new self('Invalid or revoked refresh token', 401);
    }
}

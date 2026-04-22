<?php

declare(strict_types=1);

namespace ArchivioParlante\Exception;

use RuntimeException;

/**
 * Exception thrown when input validation fails
 *
 * This exception is thrown when:
 * - Email format is invalid
 * - Password does not meet strength requirements
 * - Required fields are missing in request payload
 * - Field values do not match expected constraints
 *
 * HTTP Status Code: 400 Bad Request
 */
final class ValidationException extends RuntimeException
{
    /**
     * @var array<string, string[]> Validation errors indexed by field name
     */
    private array $errors;

    /**
     * Create a new validation exception
     *
     * @param array<string, string[]> $errors Validation errors (e.g., ['email' => ['Invalid format'], 'password' => ['Too short', 'Missing uppercase']])
     * @param string $message General error message (default: 'Validation failed')
     * @param int $code Error code (default: 400)
     * @param \Throwable|null $previous Previous exception for chaining
     */
    public function __construct(array $errors = [], string $message = 'Validation failed', int $code = 400, ?\Throwable $previous = null)
    {
        $this->errors = $errors;
        parent::__construct($message, $code, $previous);
    }

    /**
     * Get validation errors
     *
     * @return array<string, string[]> Errors indexed by field name
     */
    public function getErrors(): array
    {
        return $this->errors;
    }

    /**
     * Create exception for a single field error
     *
     * @param string $field Field name (e.g., 'email', 'password')
     * @param string $error Error message (e.g., 'Invalid format')
     */
    public static function forField(string $field, string $error): self
    {
        return new self([$field => [$error]]);
    }

    /**
     * Create exception for multiple errors
     *
     * @param array<string, string[]> $errors Errors indexed by field name
     */
    public static function withErrors(array $errors): self
    {
        return new self($errors);
    }

    /**
     * Create exception for invalid email format
     */
    public static function invalidEmail(): self
    {
        return self::forField('email', 'Invalid email format');
    }

    /**
     * Create exception for weak password
     *
     * @param string[] $reasons Specific reasons why password is weak
     */
    public static function weakPassword(array $reasons): self
    {
        return new self(['password' => $reasons]);
    }

    /**
     * Create exception for missing required field
     */
    public static function missingField(string $field): self
    {
        return self::forField($field, 'This field is required');
    }

    /**
     * Create exception for duplicate email
     */
    public static function duplicateEmail(): self
    {
        return self::forField('email', 'Email already registered');
    }
}

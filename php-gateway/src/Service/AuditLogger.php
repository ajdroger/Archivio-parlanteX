<?php

declare(strict_types=1);

namespace ArchivioParlante\Service;

use ArchivioParlante\Repository\AuditLogRepository;
use Psr\Log\LoggerInterface;

/**
 * High-level audit logging service
 *
 * Provides semantic methods for logging security events to ap_audit_logs table.
 * All methods are fire-and-forget (they never throw exceptions).
 *
 * Logged events:
 * - Authentication: register, login success/failed, logout, token refresh
 * - Authorization: unauthorized access attempts
 * - Rate limiting: exceeded request limits
 * - Account management: password changes (future)
 *
 * Note: Not declared final to allow mocking in unit tests.
 */
class AuditLogger
{
    public function __construct(
        private AuditLogRepository $auditLogRepository,
        private LoggerInterface $logger
    ) {
    }

    /**
     * Log authentication-related event
     *
     * @param string $eventType Event type (register_success, login_success, login_failed, logout, token_refresh, password_change)
     * @param int|null $userId User ID (null for failed login or anonymous events)
     * @param string $ipAddress Client IP address
     * @param string $userAgent Browser User-Agent header
     * @param string $requestPath API endpoint path
     * @param string $requestMethod HTTP method
     * @param int $statusCode HTTP status code
     * @param array<string, mixed> $eventData Additional context (e.g., failure reason)
     * @return void
     */
    public function logAuthEvent(
        string $eventType,
        ?int $userId,
        string $ipAddress,
        string $userAgent,
        string $requestPath,
        string $requestMethod,
        int $statusCode,
        array $eventData = []
    ): void {
        $this->auditLogRepository->create(
            userId: $userId,
            eventType: $eventType,
            ipAddress: $ipAddress,
            userAgent: $userAgent,
            requestPath: $requestPath,
            requestMethod: $requestMethod,
            statusCode: $statusCode,
            eventData: !empty($eventData) ? $eventData : null
        );

        $this->logger->info('Auth event logged', [
            'event_type' => $eventType,
            'user_id' => $userId,
            'ip' => $ipAddress,
            'status_code' => $statusCode,
        ]);
    }

    /**
     * Log security event (unauthorized access, token tampering, etc.)
     *
     * @param string $eventType Event type (unauthorized_access)
     * @param string $ipAddress Client IP address
     * @param string $userAgent Browser User-Agent header
     * @param string $requestPath API endpoint path
     * @param string $requestMethod HTTP method
     * @param int $statusCode HTTP status code (typically 401, 403)
     * @param array<string, mixed> $eventData Additional context
     * @return void
     */
    public function logSecurityEvent(
        string $eventType,
        string $ipAddress,
        string $userAgent,
        string $requestPath,
        string $requestMethod,
        int $statusCode,
        array $eventData = []
    ): void {
        $this->auditLogRepository->create(
            userId: null, // Security events are typically anonymous
            eventType: $eventType,
            ipAddress: $ipAddress,
            userAgent: $userAgent,
            requestPath: $requestPath,
            requestMethod: $requestMethod,
            statusCode: $statusCode,
            eventData: !empty($eventData) ? $eventData : null
        );

        $this->logger->warning('Security event logged', [
            'event_type' => $eventType,
            'ip' => $ipAddress,
            'path' => $requestPath,
            'status_code' => $statusCode,
        ]);
    }

    /**
     * Log rate limit violation
     *
     * @param string $ipAddress Client IP address
     * @param string $userAgent Browser User-Agent header
     * @param string $requestPath API endpoint path
     * @param string $requestMethod HTTP method
     * @param int $attempts Number of attempts that triggered the rate limit
     * @return void
     */
    public function logRateLimitViolation(
        string $ipAddress,
        string $userAgent,
        string $requestPath,
        string $requestMethod,
        int $attempts
    ): void {
        $this->auditLogRepository->create(
            userId: null, // Rate limit events are IP-based, not user-based
            eventType: 'rate_limit_exceeded',
            ipAddress: $ipAddress,
            userAgent: $userAgent,
            requestPath: $requestPath,
            requestMethod: $requestMethod,
            statusCode: 429, // Too Many Requests
            eventData: [
                'attempts' => $attempts,
                'message' => 'Rate limit exceeded',
            ]
        );

        $this->logger->warning('Rate limit violation logged', [
            'ip' => $ipAddress,
            'path' => $requestPath,
            'attempts' => $attempts,
        ]);
    }
}

<?php

declare(strict_types=1);

namespace ArchivioParlante\Repository;

use PDO;
use PDOException;
use Psr\Log\LoggerInterface;

/**
 * Repository for ap_audit_logs table
 *
 * Fire-and-forget pattern: INSERT operations catch exceptions internally
 * and log errors without throwing, to prevent audit logging failures
 * from breaking application flow.
 *
 * All queries use prepared statements for SQL injection prevention.
 */
class AuditLogRepository
{
    public function __construct(
        private PDO $pdo,
        private LoggerInterface $logger
    ) {
    }

    /**
     * Create audit log entry
     *
     * Fire-and-forget: Returns void and never throws exceptions.
     * Failures are logged but do not propagate to caller.
     *
     * @param int|null $userId User ID (null for anonymous events like failed login)
     * @param string $eventType Event type (must match ENUM values in database)
     * @param string $ipAddress Client IP address (IPv4 or IPv6)
     * @param string|null $userAgent Browser User-Agent header
     * @param string $requestPath API endpoint path
     * @param string $requestMethod HTTP method (GET, POST, etc.)
     * @param int $statusCode HTTP status code (200, 401, 429, etc.)
     * @param array<string, mixed>|null $eventData Additional event context (encoded as JSON)
     * @return void
     */
    public function create(
        ?int $userId,
        string $eventType,
        string $ipAddress,
        ?string $userAgent,
        string $requestPath,
        string $requestMethod,
        int $statusCode,
        ?array $eventData = null
    ): void {
        try {
            // Encode event_data as JSON if provided
            $eventDataJson = $eventData !== null ? json_encode($eventData, JSON_THROW_ON_ERROR) : null;

            $stmt = $this->pdo->prepare('
                INSERT INTO ap_audit_logs (
                    user_id,
                    event_type,
                    ip_address,
                    user_agent,
                    request_path,
                    request_method,
                    status_code,
                    event_data,
                    created_at
                ) VALUES (
                    :user_id,
                    :event_type,
                    :ip_address,
                    :user_agent,
                    :request_path,
                    :request_method,
                    :status_code,
                    :event_data,
                    CURRENT_TIMESTAMP
                )
            ');

            $stmt->execute([
                'user_id' => $userId,
                'event_type' => $eventType,
                'ip_address' => $ipAddress,
                'user_agent' => $userAgent,
                'request_path' => $requestPath,
                'request_method' => $requestMethod,
                'status_code' => $statusCode,
                'event_data' => $eventDataJson,
            ]);

            $this->logger->debug('Audit log entry created', [
                'event_type' => $eventType,
                'user_id' => $userId,
                'status_code' => $statusCode,
            ]);
        } catch (PDOException $e) {
            // Fire-and-forget: log error but don't throw
            $this->logger->error('Failed to create audit log entry', [
                'error' => $e->getMessage(),
                'event_type' => $eventType,
                'user_id' => $userId,
                'ip_address' => $ipAddress,
            ]);
        } catch (\JsonException $e) {
            // JSON encoding failed
            $this->logger->error('Failed to encode audit log event_data as JSON', [
                'error' => $e->getMessage(),
                'event_type' => $eventType,
                'event_data' => $eventData,
            ]);
        }
    }
}

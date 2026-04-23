<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Repository\AuditLogRepository;
use PHPUnit\Framework\TestCase;
use Psr\Log\NullLogger;

final class AuditLogRepositoryTest extends TestCase
{
    private \PDO $pdo;
    private AuditLogRepository $repository;

    protected function setUp(): void
    {
        // Use in-memory SQLite for testing
        $this->pdo = new \PDO('sqlite::memory:');
        $this->pdo->setAttribute(\PDO::ATTR_ERRMODE, \PDO::ERRMODE_EXCEPTION);

        // Create simplified ap_audit_logs table (SQLite doesn't support ENUM)
        $this->pdo->exec('
            CREATE TABLE ap_audit_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER,
                event_type TEXT NOT NULL,
                ip_address TEXT NOT NULL,
                user_agent TEXT,
                request_path TEXT NOT NULL,
                request_method TEXT NOT NULL,
                status_code INTEGER NOT NULL,
                event_data TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
        ');

        $this->repository = new AuditLogRepository($this->pdo, new NullLogger());
    }

    public function testCreateInsertsAuditLogSuccessfully(): void
    {
        $this->repository->create(
            userId: 123,
            eventType: 'login_success',
            ipAddress: '192.168.1.1',
            userAgent: 'Mozilla/5.0',
            requestPath: '/api/auth/login',
            requestMethod: 'POST',
            statusCode: 200
        );

        $stmt = $this->pdo->query('SELECT * FROM ap_audit_logs');
        $logs = $stmt->fetchAll(\PDO::FETCH_ASSOC);

        $this->assertCount(1, $logs);
        $this->assertSame(123, $logs[0]['user_id']);
        $this->assertSame('login_success', $logs[0]['event_type']);
        $this->assertSame('192.168.1.1', $logs[0]['ip_address']);
        $this->assertSame('Mozilla/5.0', $logs[0]['user_agent']);
        $this->assertSame('/api/auth/login', $logs[0]['request_path']);
        $this->assertSame('POST', $logs[0]['request_method']);
        $this->assertSame(200, $logs[0]['status_code']);
        $this->assertNull($logs[0]['event_data']);
    }

    public function testCreateAcceptsNullUserId(): void
    {
        $this->repository->create(
            userId: null,
            eventType: 'login_failed',
            ipAddress: '10.0.0.1',
            userAgent: 'curl/7.68.0',
            requestPath: '/api/auth/login',
            requestMethod: 'POST',
            statusCode: 401
        );

        $stmt = $this->pdo->query('SELECT * FROM ap_audit_logs');
        $logs = $stmt->fetchAll(\PDO::FETCH_ASSOC);

        $this->assertCount(1, $logs);
        $this->assertNull($logs[0]['user_id']);
        $this->assertSame('login_failed', $logs[0]['event_type']);
    }

    public function testCreateStoresEventDataAsJson(): void
    {
        $eventData = ['reason' => 'invalid_password', 'attempts' => 3];

        $this->repository->create(
            userId: 456,
            eventType: 'login_failed',
            ipAddress: '172.16.0.1',
            userAgent: null,
            requestPath: '/api/auth/login',
            requestMethod: 'POST',
            statusCode: 401,
            eventData: $eventData
        );

        $stmt = $this->pdo->query('SELECT * FROM ap_audit_logs');
        $logs = $stmt->fetchAll(\PDO::FETCH_ASSOC);

        $this->assertCount(1, $logs);
        $decoded = json_decode($logs[0]['event_data'], true);
        $this->assertSame('invalid_password', $decoded['reason']);
        $this->assertSame(3, $decoded['attempts']);
    }

    public function testCreateDoesNotThrowOnPdoException(): void
    {
        // Drop table to force PDOException
        $this->pdo->exec('DROP TABLE ap_audit_logs');

        // Fire-and-forget: should not throw
        $this->repository->create(
            userId: 1,
            eventType: 'test',
            ipAddress: '127.0.0.1',
            userAgent: 'test',
            requestPath: '/test',
            requestMethod: 'GET',
            statusCode: 200
        );

        // If we reach here, test passes (no exception thrown)
        $this->assertTrue(true);
    }

    public function testCreateHandlesIpv6Addresses(): void
    {
        $ipv6 = '2001:0db8:85a3:0000:0000:8a2e:0370:7334';

        $this->repository->create(
            userId: 789,
            eventType: 'register_success',
            ipAddress: $ipv6,
            userAgent: 'Firefox/95.0',
            requestPath: '/api/auth/register',
            requestMethod: 'POST',
            statusCode: 201
        );

        $stmt = $this->pdo->query('SELECT * FROM ap_audit_logs');
        $logs = $stmt->fetchAll(\PDO::FETCH_ASSOC);

        $this->assertCount(1, $logs);
        $this->assertSame($ipv6, $logs[0]['ip_address']);
    }
}

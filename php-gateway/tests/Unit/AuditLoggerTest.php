<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Repository\AuditLogRepository;
use ArchivioParlante\Service\AuditLogger;
use PHPUnit\Framework\TestCase;
use PHPUnit\Framework\MockObject\MockObject;
use Psr\Log\NullLogger;

final class AuditLoggerTest extends TestCase
{
    private AuditLogger $auditLogger;
    private AuditLogRepository & MockObject $repository;

    protected function setUp(): void
    {
        $this->repository = $this->createMock(AuditLogRepository::class);
        $this->auditLogger = new AuditLogger($this->repository, new NullLogger());
    }

    public function testLogAuthEventCallsRepositoryWithCorrectParameters(): void
    {
        $this->repository->expects($this->once())
            ->method('create')
            ->with(
                123,
                'login_success',
                '192.168.1.1',
                'Mozilla/5.0',
                '/api/auth/login',
                'POST',
                200,
                null
            );

        $this->auditLogger->logAuthEvent(
            eventType: 'login_success',
            userId: 123,
            ipAddress: '192.168.1.1',
            userAgent: 'Mozilla/5.0',
            requestPath: '/api/auth/login',
            requestMethod: 'POST',
            statusCode: 200
        );
    }

    public function testLogAuthEventPassesEventDataToRepository(): void
    {
        $eventData = ['reason' => 'invalid_password', 'attempts' => 3];

        $this->repository->expects($this->once())
            ->method('create')
            ->with(
                456,
                'login_failed',
                '10.0.0.1',
                'curl/7.68.0',
                '/api/auth/login',
                'POST',
                401,
                $eventData
            );

        $this->auditLogger->logAuthEvent(
            eventType: 'login_failed',
            userId: 456,
            ipAddress: '10.0.0.1',
            userAgent: 'curl/7.68.0',
            requestPath: '/api/auth/login',
            requestMethod: 'POST',
            statusCode: 401,
            eventData: $eventData
        );
    }

    public function testLogAuthEventAcceptsNullUserId(): void
    {
        $this->repository->expects($this->once())
            ->method('create')
            ->with(
                null,
                'login_failed',
                '172.16.0.1',
                'Unknown',
                '/api/auth/login',
                'POST',
                401,
                ['reason' => 'user_not_found']
            );

        $this->auditLogger->logAuthEvent(
            eventType: 'login_failed',
            userId: null,
            ipAddress: '172.16.0.1',
            userAgent: 'Unknown',
            requestPath: '/api/auth/login',
            requestMethod: 'POST',
            statusCode: 401,
            eventData: ['reason' => 'user_not_found']
        );
    }

    public function testLogSecurityEventCallsRepositoryWithNullUserId(): void
    {
        $this->repository->expects($this->once())
            ->method('create')
            ->with(
                null,
                'unauthorized_access',
                '203.0.113.42',
                'Postman',
                '/api/auth/me',
                'GET',
                401,
                null
            );

        $this->auditLogger->logSecurityEvent(
            eventType: 'unauthorized_access',
            ipAddress: '203.0.113.42',
            userAgent: 'Postman',
            requestPath: '/api/auth/me',
            requestMethod: 'GET',
            statusCode: 401
        );
    }

    public function testLogSecurityEventPassesEventData(): void
    {
        $eventData = ['reason' => 'missing_token', 'path' => '/api/admin'];

        $this->repository->expects($this->once())
            ->method('create')
            ->with(
                null,
                'unauthorized_access',
                '198.51.100.10',
                'Firefox',
                '/api/admin',
                'GET',
                403,
                $eventData
            );

        $this->auditLogger->logSecurityEvent(
            eventType: 'unauthorized_access',
            ipAddress: '198.51.100.10',
            userAgent: 'Firefox',
            requestPath: '/api/admin',
            requestMethod: 'GET',
            statusCode: 403,
            eventData: $eventData
        );
    }

    public function testLogRateLimitViolationCallsRepositoryWithCorrectParameters(): void
    {
        $this->repository->expects($this->once())
            ->method('create')
            ->with(
                null,
                'rate_limit_exceeded',
                '192.0.2.1',
                'Chrome/95.0',
                '/api/auth/login',
                'POST',
                429,
                [
                    'attempts' => 6,
                    'message' => 'Rate limit exceeded',
                ]
            );

        $this->auditLogger->logRateLimitViolation(
            ipAddress: '192.0.2.1',
            userAgent: 'Chrome/95.0',
            requestPath: '/api/auth/login',
            requestMethod: 'POST',
            attempts: 6
        );
    }
}

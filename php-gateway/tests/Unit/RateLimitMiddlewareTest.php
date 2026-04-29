<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Middleware\RateLimitMiddleware;
use ArchivioParlante\Service\AuditLogger;
use ArchivioParlante\Service\RedisSessionManager;
use PHPUnit\Framework\TestCase;
use PHPUnit\Framework\MockObject\MockObject;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Message\UriInterface;
use Psr\Http\Server\RequestHandlerInterface;
use Psr\Log\NullLogger;
use Slim\Psr7\Response;

final class RateLimitMiddlewareTest extends TestCase
{
    private RedisSessionManager & MockObject $sessionManager;
    private AuditLogger & MockObject $auditLogger;
    private ServerRequestInterface & MockObject $request;
    private RequestHandlerInterface & MockObject $handler;
    private UriInterface & MockObject $uri;

    protected function setUp(): void
    {
        $this->sessionManager = $this->createMock(RedisSessionManager::class);
        $this->auditLogger = $this->createMock(AuditLogger::class);
        $this->request = $this->createMock(ServerRequestInterface::class);
        $this->handler = $this->createMock(RequestHandlerInterface::class);
        $this->uri = $this->createMock(UriInterface::class);
    }

    public function testProcessAllowsRequestWhenUnderLimit(): void
    {
        $middleware = new RateLimitMiddleware(
            $this->sessionManager,
            $this->auditLogger,
            new NullLogger(),
            maxAttempts: 5,
            windowSeconds: 900,
            endpointName: 'login'
        );

        $this->request->method('getServerParams')->willReturn(['REMOTE_ADDR' => '192.168.1.1']);
        $this->request->method('getHeaderLine')->willReturn('curl/7.68.0');
        $this->request->method('getUri')->willReturn($this->uri);
        $this->request->method('getMethod')->willReturn('POST');
        $this->uri->method('getPath')->willReturn('/api/auth/login');

        $this->sessionManager->expects($this->once())
            ->method('getRateLimitAttempts')
            ->with('ratelimit:login:192.168.1.1')
            ->willReturn(3);

        $expectedResponse = new Response();
        $this->handler->expects($this->once())
            ->method('handle')
            ->with($this->request)
            ->willReturn($expectedResponse);

        $response = $middleware->process($this->request, $this->handler);

        $this->assertSame($expectedResponse, $response);
    }

    public function testProcessReturns429WhenLimitExceeded(): void
    {
        $middleware = new RateLimitMiddleware(
            $this->sessionManager,
            $this->auditLogger,
            new NullLogger(),
            maxAttempts: 5,
            windowSeconds: 900,
            endpointName: 'login'
        );

        $this->request->method('getServerParams')->willReturn(['REMOTE_ADDR' => '10.0.0.1']);
        $this->request->method('getHeaderLine')->willReturn('Mozilla/5.0');
        $this->request->method('getUri')->willReturn($this->uri);
        $this->request->method('getMethod')->willReturn('POST');
        $this->uri->method('getPath')->willReturn('/api/auth/login');

        $this->sessionManager->expects($this->once())
            ->method('getRateLimitAttempts')
            ->with('ratelimit:login:10.0.0.1')
            ->willReturn(6);

        $this->auditLogger->expects($this->once())
            ->method('logRateLimitViolation')
            ->with(
                '10.0.0.1',
                'Mozilla/5.0',
                '/api/auth/login',
                'POST',
                6
            );

        $this->handler->expects($this->never())->method('handle');

        $response = $middleware->process($this->request, $this->handler);

        $this->assertSame(429, $response->getStatusCode());
        $this->assertSame('application/json', $response->getHeaderLine('Content-Type'));
        $this->assertSame('900', $response->getHeaderLine('Retry-After'));

        $body = (string) $response->getBody();
        $data = json_decode($body, true);
        $this->assertSame('Too many requests', $data['error']);
        $this->assertStringContainsString('Maximum 5 requests exceeded', $data['message']);
    }

    public function testConfigurableLimitsForRegisterEndpoint(): void
    {
        $middleware = new RateLimitMiddleware(
            $this->sessionManager,
            $this->auditLogger,
            new NullLogger(),
            maxAttempts: 3,
            windowSeconds: 3600,
            endpointName: 'register'
        );

        $this->request->method('getServerParams')->willReturn(['REMOTE_ADDR' => '172.16.0.1']);
        $this->request->method('getHeaderLine')->willReturn('Chrome/95.0');
        $this->request->method('getUri')->willReturn($this->uri);
        $this->request->method('getMethod')->willReturn('POST');
        $this->uri->method('getPath')->willReturn('/api/auth/register');

        $this->sessionManager->expects($this->once())
            ->method('getRateLimitAttempts')
            ->with('ratelimit:register:172.16.0.1')
            ->willReturn(4);

        $this->auditLogger->expects($this->once())
            ->method('logRateLimitViolation');

        $response = $middleware->process($this->request, $this->handler);

        $this->assertSame(429, $response->getStatusCode());
        $body = (string) $response->getBody();
        $data = json_decode($body, true);
        $this->assertStringContainsString('Maximum 3 requests exceeded', $data['message']);
        $this->assertStringContainsString('1 hour', $data['message']);
        $this->assertSame(3600, $data['retry_after']);
    }

    public function testConfigurableLimitsForRefreshEndpoint(): void
    {
        $middleware = new RateLimitMiddleware(
            $this->sessionManager,
            $this->auditLogger,
            new NullLogger(),
            maxAttempts: 20,
            windowSeconds: 900,
            endpointName: 'refresh'
        );

        $this->request->method('getServerParams')->willReturn(['REMOTE_ADDR' => '198.51.100.1']);
        $this->request->method('getHeaderLine')->willReturn('Postman');
        $this->request->method('getUri')->willReturn($this->uri);
        $this->request->method('getMethod')->willReturn('POST');
        $this->uri->method('getPath')->willReturn('/api/auth/refresh');

        $this->sessionManager->expects($this->once())
            ->method('getRateLimitAttempts')
            ->with('ratelimit:refresh:198.51.100.1')
            ->willReturn(19);

        $expectedResponse = new Response();
        $this->handler->expects($this->once())
            ->method('handle')
            ->willReturn($expectedResponse);

        $response = $middleware->process($this->request, $this->handler);

        $this->assertSame($expectedResponse, $response);
    }

    public function testHandlesDefaultIpWhenRemoteAddrMissing(): void
    {
        $middleware = new RateLimitMiddleware(
            $this->sessionManager,
            $this->auditLogger,
            new NullLogger(),
            maxAttempts: 5,
            windowSeconds: 900,
            endpointName: 'login'
        );

        $this->request->method('getServerParams')->willReturn([]);
        $this->request->method('getHeaderLine')->willReturn('');
        $this->request->method('getUri')->willReturn($this->uri);
        $this->request->method('getMethod')->willReturn('POST');
        $this->uri->method('getPath')->willReturn('/api/auth/login');

        $this->sessionManager->expects($this->once())
            ->method('getRateLimitAttempts')
            ->with('ratelimit:login:127.0.0.1')
            ->willReturn(2);

        $expectedResponse = new Response();
        $this->handler->expects($this->once())
            ->method('handle')
            ->willReturn($expectedResponse);

        $response = $middleware->process($this->request, $this->handler);

        $this->assertSame($expectedResponse, $response);
    }
}

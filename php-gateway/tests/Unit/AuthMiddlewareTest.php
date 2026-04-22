<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Exception\AuthenticationException;
use ArchivioParlante\Middleware\AuthMiddleware;
use ArchivioParlante\Service\JwtService;
use PHPUnit\Framework\TestCase;
use PHPUnit\Framework\MockObject\MockObject;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Server\RequestHandlerInterface;
use Psr\Log\NullLogger;
use Slim\Psr7\Response;

final class AuthMiddlewareTest extends TestCase
{
    private AuthMiddleware $middleware;
    private JwtService & MockObject $jwtService;
    private ServerRequestInterface & MockObject $request;
    private RequestHandlerInterface & MockObject $handler;

    protected function setUp(): void
    {
        $this->jwtService = $this->createMock(JwtService::class);
        $this->middleware = new AuthMiddleware($this->jwtService, new NullLogger());

        $this->request = $this->createMock(ServerRequestInterface::class);
        $this->handler = $this->createMock(RequestHandlerInterface::class);
    }

    public function testProcessSucceedsWithValidToken(): void
    {
        $payload = [
            'sub' => '123',
            'email' => 'test@example.com',
            'role' => 'user',
            'iat' => time(),
            'exp' => time() + 900,
        ];

        $this->request->expects($this->once())
            ->method('getHeaderLine')
            ->with('Authorization')
            ->willReturn('Bearer valid-token');

        $this->jwtService->expects($this->once())
            ->method('extractTokenFromHeader')
            ->with('Bearer valid-token')
            ->willReturn('valid-token');

        $this->jwtService->expects($this->once())
            ->method('validateAccessToken')
            ->with('valid-token')
            ->willReturn($payload);

        $this->request->expects($this->once())
            ->method('withAttribute')
            ->with('user', $payload)
            ->willReturnSelf();

        $this->handler->expects($this->once())
            ->method('handle')
            ->with($this->request)
            ->willReturn(new Response());

        $response = $this->middleware->process($this->request, $this->handler);

        $this->assertInstanceOf(\Psr\Http\Message\ResponseInterface::class, $response);
    }

    public function testProcessReturns401ForMissingAuthHeader(): void
    {
        $this->request->expects($this->once())
            ->method('getHeaderLine')
            ->with('Authorization')
            ->willReturn('');

        $response = $this->middleware->process($this->request, $this->handler);

        $this->assertSame(401, $response->getStatusCode());
        $this->assertStringContainsString('Unauthorized', (string) $response->getBody());
    }

    public function testProcessReturns401ForExpiredToken(): void
    {
        $this->request->expects($this->once())
            ->method('getHeaderLine')
            ->with('Authorization')
            ->willReturn('Bearer expired-token');

        $this->jwtService->expects($this->once())
            ->method('extractTokenFromHeader')
            ->willReturn('expired-token');

        $this->jwtService->expects($this->once())
            ->method('validateAccessToken')
            ->willThrowException(AuthenticationException::invalidToken('Access token expired'));

        $response = $this->middleware->process($this->request, $this->handler);

        $this->assertSame(401, $response->getStatusCode());
    }

    public function testProcessReturns401ForMalformedToken(): void
    {
        $this->request->expects($this->once())
            ->method('getHeaderLine')
            ->with('Authorization')
            ->willReturn('InvalidFormat');

        $this->jwtService->expects($this->once())
            ->method('extractTokenFromHeader')
            ->willReturn(null);

        $response = $this->middleware->process($this->request, $this->handler);

        $this->assertSame(401, $response->getStatusCode());
    }
}

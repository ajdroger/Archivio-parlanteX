<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Controller\HealthController;
use ArchivioParlante\Service\RustEngineProxy;
use PHPUnit\Framework\TestCase;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Message\StreamInterface;
use Psr\Log\LoggerInterface;

final class HealthControllerTest extends TestCase
{
    public function testHealthReturnsOkWhenRustEngineConnected(): void
    {
        // Mock dependencies
        $logger = $this->createMock(LoggerInterface::class);

        $rustProxy = $this->createMock(RustEngineProxy::class);
        $rustProxy->method('checkHealth')
            ->willReturn(['status' => 'ok', 'service' => 'rust-engine']);

        $controller = new HealthController($logger, $rustProxy);

        // Mock request and response
        $request = $this->createMock(ServerRequestInterface::class);

        $body = $this->createMock(StreamInterface::class);
        $body->expects($this->once())
            ->method('write')
            ->with($this->callback(function ($json) {
                $data = json_decode($json, true);
                return $data['status'] === 'ok'
                    && $data['service'] === 'php-gateway'
                    && $data['rust_engine'] === 'connected';
            }));

        $response = $this->createMock(ResponseInterface::class);
        $response->method('getBody')->willReturn($body);
        $response->method('withHeader')->willReturnSelf();
        $response->method('withStatus')->willReturnSelf();

        // Execute
        $result = $controller->health($request, $response);

        $this->assertInstanceOf(ResponseInterface::class, $result);
    }

    public function testHealthReturnsUnreachableWhenRustEngineFails(): void
    {
        $logger = $this->createMock(LoggerInterface::class);
        $logger->expects($this->once())->method('warning');

        $rustProxy = $this->createMock(RustEngineProxy::class);
        $rustProxy->method('checkHealth')
            ->willThrowException(new \RuntimeException('Connection failed'));

        $controller = new HealthController($logger, $rustProxy);

        $request = $this->createMock(ServerRequestInterface::class);

        $body = $this->createMock(StreamInterface::class);
        $body->expects($this->once())
            ->method('write')
            ->with($this->callback(function ($json) {
                $data = json_decode($json, true);
                return $data['rust_engine'] === 'unreachable';
            }));

        $response = $this->createMock(ResponseInterface::class);
        $response->method('getBody')->willReturn($body);
        $response->method('withHeader')->willReturnSelf();
        $response->method('withStatus')->willReturnSelf();

        $controller->health($request, $response);
    }
}

<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Service\RustEngineProxy;
use GuzzleHttp\Client;
use GuzzleHttp\Exception\ConnectException;
use GuzzleHttp\Psr7\Request;
use GuzzleHttp\Psr7\Response;
use PHPUnit\Framework\TestCase;
use Psr\Log\LoggerInterface;

final class RustEngineProxyTest extends TestCase
{
    public function testCheckHealthReturnsDataOnSuccess(): void
    {
        $logger = $this->createMock(LoggerInterface::class);

        $httpClient = $this->createMock(Client::class);
        $httpClient->method('get')
            ->willReturn(new Response(200, [], '{"status": "ok", "service": "rust-engine"}'));

        $proxy = new RustEngineProxy($httpClient, $logger, 'http://localhost:8090', '');

        $result = $proxy->checkHealth();

        $this->assertSame('ok', $result['status']);
        $this->assertSame('rust-engine', $result['service']);
    }

    public function testCheckHealthThrowsOnConnectionFailure(): void
    {
        $logger = $this->createMock(LoggerInterface::class);
        $logger->expects($this->once())->method('error');

        $httpClient = $this->createMock(Client::class);
        $httpClient->method('get')
            ->willThrowException(new ConnectException(
                'Connection refused',
                new Request('GET', 'http://localhost:8090/health')
            ));

        $proxy = new RustEngineProxy($httpClient, $logger, 'http://localhost:8090', '');

        $this->expectException(\RuntimeException::class);
        $this->expectExceptionMessage('Failed to connect to Rust engine');

        $proxy->checkHealth();
    }

    public function testProxyRequestAddsInternalToken(): void
    {
        $logger = $this->createMock(LoggerInterface::class);

        $httpClient = $this->createMock(Client::class);
        $httpClient->expects($this->once())
            ->method('request')
            ->with(
                'POST',
                'http://localhost:8090/query',
                $this->callback(function ($options) {
                    return isset($options['headers']['X-Internal-Token'])
                        && $options['headers']['X-Internal-Token'] === 'secret-token';
                })
            )
            ->willReturn(new Response(200, [], '{"results": []}'));

        $proxy = new RustEngineProxy($httpClient, $logger, 'http://localhost:8090', 'secret-token');

        $result = $proxy->proxyRequest('POST', '/query', ['query' => 'test']);

        $this->assertIsArray($result);
    }

    public function testProxyRequestThrowsOn4xxError(): void
    {
        $logger = $this->createMock(LoggerInterface::class);
        $logger->expects($this->once())->method('warning');

        $httpClient = $this->createMock(Client::class);
        $httpClient->method('request')
            ->willReturn(new Response(400, [], '{"error": "Bad Request"}'));

        $proxy = new RustEngineProxy($httpClient, $logger, 'http://localhost:8090', '');

        $this->expectException(\RuntimeException::class);
        $this->expectExceptionMessage('Rust engine error (400): Bad Request');

        $proxy->proxyRequest('POST', '/query', []);
    }
}

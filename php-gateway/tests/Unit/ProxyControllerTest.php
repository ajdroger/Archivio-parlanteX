<?php

declare(strict_types=1);

namespace ArchivioParlante\Tests\Unit;

use ArchivioParlante\Controller\ProxyController;
use ArchivioParlante\Service\RustEngineProxy;
use ArchivioParlante\Service\AuditLogger;
use ArchivioParlante\Exception\ValidationException;
use PHPUnit\Framework\TestCase;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Message\StreamInterface;
use Psr\Log\LoggerInterface;

class ProxyControllerTest extends TestCase
{
    private RustEngineProxy $rustEngine;
    private AuditLogger $auditLogger;
    private LoggerInterface $logger;
    private ProxyController $controller;

    protected function setUp(): void
    {
        $this->rustEngine = $this->createMock(RustEngineProxy::class);
        $this->auditLogger = $this->createMock(AuditLogger::class);
        $this->logger = $this->createMock(LoggerInterface::class);

        $this->controller = new ProxyController(
            $this->rustEngine,
            $this->auditLogger,
            $this->logger
        );
    }

    public function testQuerySuccess(): void
    {
        // Arrange
        $user = ['id' => 1, 'email' => 'test@example.com'];
        $requestBody = [
            'kb_id' => 'test_kb',
            'query' => 'Test query',
            'top_k' => 10,
        ];
        $rustResponse = [
            'answer' => 'Test answer',
            'sources' => [],
        ];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);
        $stream = $this->createMock(StreamInterface::class);
        $response->method('getBody')->willReturn($stream);
        $response->method('withHeader')->willReturnSelf();
        $response->method('withStatus')->willReturnSelf();

        $this->rustEngine
            ->expects($this->once())
            ->method('query')
            ->with($requestBody)
            ->willReturn($rustResponse);

        $this->auditLogger
            ->expects($this->once())
            ->method('logEvent')
            ->with(
                'query_success',
                1,
                $this->anything(),
                '/api/query',
                'POST',
                200,
                $this->arrayHasKey('kb_id')
            );

        // Act
        $result = $this->controller->query($request, $response);

        // Assert
        $this->assertSame($response, $result);
    }

    public function testQueryValidationFailsWithoutKbId(): void
    {
        $this->expectException(ValidationException::class);
        $this->expectExceptionMessage('Field "kb_id" is required');

        $user = ['id' => 1];
        $requestBody = ['query' => 'Test query'];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);

        $this->controller->query($request, $response);
    }

    public function testQueryValidationFailsWithoutQuery(): void
    {
        $this->expectException(ValidationException::class);
        $this->expectExceptionMessage('Field "query" is required');

        $user = ['id' => 1];
        $requestBody = ['kb_id' => 'test_kb'];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);

        $this->controller->query($request, $response);
    }

    public function testQueryValidationFailsWithLongQuery(): void
    {
        $this->expectException(ValidationException::class);
        $this->expectExceptionMessage('Query exceeds maximum length');

        $user = ['id' => 1];
        $requestBody = [
            'kb_id' => 'test_kb',
            'query' => str_repeat('x', 1001),
        ];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);

        $this->controller->query($request, $response);
    }

    public function testIngestSuccess(): void
    {
        // Arrange
        $user = ['id' => 1];
        $requestBody = [
            'doc_id' => 'doc_001',
            'kb_id' => 'test_kb',
            'file_path' => '/shared/uploads/test.pdf',
            'mime_type' => 'application/pdf',
        ];
        $rustResponse = [
            'chunk_count' => 42,
            'processing_time_ms' => 1500,
        ];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);
        $stream = $this->createMock(StreamInterface::class);
        $response->method('getBody')->willReturn($stream);
        $response->method('withHeader')->willReturnSelf();
        $response->method('withStatus')->willReturnSelf();

        $this->rustEngine
            ->expects($this->once())
            ->method('ingest')
            ->with($requestBody)
            ->willReturn($rustResponse);

        // Act
        $result = $this->controller->ingest($request, $response);

        // Assert
        $this->assertSame($response, $result);
    }

    public function testIngestValidationFailsWithInvalidMimeType(): void
    {
        $this->expectException(ValidationException::class);
        $this->expectExceptionMessage('MIME type not supported');

        $user = ['id' => 1];
        $requestBody = [
            'doc_id' => 'doc_001',
            'kb_id' => 'test_kb',
            'file_path' => '/shared/uploads/test.exe',
            'mime_type' => 'application/x-msdownload',
        ];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);

        $this->controller->ingest($request, $response);
    }

    public function testCompareSuccess(): void
    {
        // Arrange
        $user = ['id' => 1];
        $requestBody = [
            'kb_id' => 'test_kb',
            'doc_ids' => ['doc_001', 'doc_002'],
            'comparison_aspects' => ['penalties', 'payment_terms'],
        ];
        $rustResponse = [
            'comparison_matrix' => [],
            'summary' => 'Comparison summary',
        ];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);
        $stream = $this->createMock(StreamInterface::class);
        $response->method('getBody')->willReturn($stream);
        $response->method('withHeader')->willReturnSelf();
        $response->method('withStatus')->willReturnSelf();

        $this->rustEngine
            ->expects($this->once())
            ->method('compare')
            ->with($requestBody)
            ->willReturn($rustResponse);

        // Act
        $result = $this->controller->compare($request, $response);

        // Assert
        $this->assertSame($response, $result);
    }

    public function testCompareValidationFailsWithTooFewDocuments(): void
    {
        $this->expectException(ValidationException::class);
        $this->expectExceptionMessage('At least 2 documents are required');

        $user = ['id' => 1];
        $requestBody = [
            'kb_id' => 'test_kb',
            'doc_ids' => ['doc_001'],
            'comparison_aspects' => ['penalties'],
        ];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);

        $this->controller->compare($request, $response);
    }

    public function testCompareValidationFailsWithTooManyDocuments(): void
    {
        $this->expectException(ValidationException::class);
        $this->expectExceptionMessage('Maximum 10 documents allowed');

        $user = ['id' => 1];
        $requestBody = [
            'kb_id' => 'test_kb',
            'doc_ids' => array_map(fn($i) => "doc_$i", range(1, 11)),
            'comparison_aspects' => ['penalties'],
        ];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);

        $this->controller->compare($request, $response);
    }

    public function testQueryHandlesRustEngineError(): void
    {
        // Arrange
        $user = ['id' => 1];
        $requestBody = [
            'kb_id' => 'test_kb',
            'query' => 'Test query',
        ];

        $request = $this->createMock(ServerRequestInterface::class);
        $request->method('getAttribute')->with('user')->willReturn($user);
        $request->method('getParsedBody')->willReturn($requestBody);

        $response = $this->createMock(ResponseInterface::class);
        $stream = $this->createMock(StreamInterface::class);
        $response->method('getBody')->willReturn($stream);
        $response->method('withHeader')->willReturnSelf();
        $response->method('withStatus')->willReturnSelf();

        $this->rustEngine
            ->method('query')
            ->willThrowException(new \RuntimeException('Rust engine unavailable'));

        $this->auditLogger
            ->expects($this->once())
            ->method('logEvent')
            ->with(
                'query_failed',
                1,
                $this->anything(),
                '/api/query',
                'POST',
                500,
                $this->arrayHasKey('error')
            );

        // Act
        $result = $this->controller->query($request, $response);

        // Assert
        $this->assertSame($response, $result);
    }
}

<?php

declare(strict_types=1);

namespace ArchivioParlante\Controller;

use ArchivioParlante\Service\RustEngineProxy;
use ArchivioParlante\Service\AuditLogger;
use ArchivioParlante\Exception\ValidationException;
use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Log\LoggerInterface;

/**
 * Proxy Controller - Forwards requests to Rust engine
 *
 * Handles /api/query, /api/ingest, /api/compare endpoints
 * All routes are protected by AuthMiddleware (JWT required)
 */
final class ProxyController
{
    public function __construct(
        private RustEngineProxy $rustEngine,
        private AuditLogger $auditLogger,
        private LoggerInterface $logger
    ) {
    }

    /**
     * POST /api/query - RAG query endpoint
     *
     * Forwards query request to Rust engine's /query endpoint
     * Performs hybrid search + reranking + LLM response generation
     *
     * Request body:
     * {
     *   "kb_id": "contracts_2024",
     *   "query": "Quali sono le penali previste?",
     *   "top_k": 10,
     *   "rerank_top_n": 5
     * }
     *
     * @param Request $request
     * @param Response $response
     * @return Response
     */
    public function query(Request $request, Response $response): Response
    {
        $user = $request->getAttribute('user');
        $body = $request->getParsedBody();

        // Validate required fields
        $this->validateQueryRequest($body);

        $this->logger->info('Proxying query request to Rust engine', [
            'user_id' => $user['id'],
            'kb_id' => $body['kb_id'] ?? null,
        ]);

        try {
            $result = $this->rustEngine->query($body);

            $this->auditLogger->logEvent(
                'query_success',
                $user['id'],
                $_SERVER['REMOTE_ADDR'] ?? '127.0.0.1',
                '/api/query',
                'POST',
                200,
                ['kb_id' => $body['kb_id'], 'query_length' => strlen($body['query'])]
            );

            $response->getBody()->write(json_encode($result));
            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(200);
        } catch (\RuntimeException $e) {
            $this->logger->error('Rust engine query failed', [
                'error' => $e->getMessage(),
                'user_id' => $user['id'],
            ]);

            $this->auditLogger->logEvent(
                'query_failed',
                $user['id'],
                $_SERVER['REMOTE_ADDR'] ?? '127.0.0.1',
                '/api/query',
                'POST',
                500,
                ['error' => $e->getMessage()]
            );

            $response->getBody()->write(json_encode([
                'error' => 'Query failed',
                'message' => $e->getMessage(),
            ]));
            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(500);
        }
    }

    /**
     * POST /api/ingest - Document ingestion endpoint
     *
     * Forwards ingest request to Rust engine's /ingest endpoint
     * Triggers document parsing, chunking, embedding, and storage pipeline
     *
     * Request body:
     * {
     *   "doc_id": "contract_001",
     *   "kb_id": "contracts_2024",
     *   "file_path": "/shared/uploads/contract_001.pdf",
     *   "mime_type": "application/pdf"
     * }
     *
     * @param Request $request
     * @param Response $response
     * @return Response
     */
    public function ingest(Request $request, Response $response): Response
    {
        $user = $request->getAttribute('user');
        $body = $request->getParsedBody();

        // Validate required fields
        $this->validateIngestRequest($body);

        $this->logger->info('Proxying ingest request to Rust engine', [
            'user_id' => $user['id'],
            'doc_id' => $body['doc_id'] ?? null,
            'kb_id' => $body['kb_id'] ?? null,
        ]);

        try {
            $result = $this->rustEngine->ingest($body);

            $this->auditLogger->logEvent(
                'ingest_success',
                $user['id'],
                $_SERVER['REMOTE_ADDR'] ?? '127.0.0.1',
                '/api/ingest',
                'POST',
                200,
                ['doc_id' => $body['doc_id'], 'kb_id' => $body['kb_id']]
            );

            $response->getBody()->write(json_encode($result));
            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(200);
        } catch (\RuntimeException $e) {
            $this->logger->error('Rust engine ingest failed', [
                'error' => $e->getMessage(),
                'user_id' => $user['id'],
            ]);

            $this->auditLogger->logEvent(
                'ingest_failed',
                $user['id'],
                $_SERVER['REMOTE_ADDR'] ?? '127.0.0.1',
                '/api/ingest',
                'POST',
                500,
                ['error' => $e->getMessage()]
            );

            $response->getBody()->write(json_encode([
                'error' => 'Ingest failed',
                'message' => $e->getMessage(),
            ]));
            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(500);
        }
    }

    /**
     * POST /api/compare - Multi-contract comparison endpoint
     *
     * Forwards comparison request to Rust engine's /compare_contracts endpoint
     * Performs parallel queries across multiple contracts and generates comparative analysis
     *
     * Request body:
     * {
     *   "kb_id": "contracts_2024",
     *   "doc_ids": ["contract_001", "contract_002", "contract_003"],
     *   "comparison_aspects": ["penalties", "payment_terms", "termination_clauses"]
     * }
     *
     * @param Request $request
     * @param Response $response
     * @return Response
     */
    public function compare(Request $request, Response $response): Response
    {
        $user = $request->getAttribute('user');
        $body = $request->getParsedBody();

        // Validate required fields
        $this->validateCompareRequest($body);

        $this->logger->info('Proxying compare request to Rust engine', [
            'user_id' => $user['id'],
            'kb_id' => $body['kb_id'] ?? null,
            'doc_count' => count($body['doc_ids'] ?? []),
        ]);

        try {
            $result = $this->rustEngine->compare($body);

            $this->auditLogger->logEvent(
                'compare_success',
                $user['id'],
                $_SERVER['REMOTE_ADDR'] ?? '127.0.0.1',
                '/api/compare',
                'POST',
                200,
                ['kb_id' => $body['kb_id'], 'doc_count' => count($body['doc_ids'])]
            );

            $response->getBody()->write(json_encode($result));
            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(200);
        } catch (\RuntimeException $e) {
            $this->logger->error('Rust engine compare failed', [
                'error' => $e->getMessage(),
                'user_id' => $user['id'],
            ]);

            $this->auditLogger->logEvent(
                'compare_failed',
                $user['id'],
                $_SERVER['REMOTE_ADDR'] ?? '127.0.0.1',
                '/api/compare',
                'POST',
                500,
                ['error' => $e->getMessage()]
            );

            $response->getBody()->write(json_encode([
                'error' => 'Compare failed',
                'message' => $e->getMessage(),
            ]));
            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(500);
        }
    }

    /**
     * Validate query request body
     *
     * @param mixed $body
     * @throws ValidationException
     */
    private function validateQueryRequest($body): void
    {
        if (!is_array($body)) {
            throw new ValidationException(['body' => ['Request body must be a JSON object']]);
        }

        if (empty($body['kb_id'])) {
            throw new ValidationException(['kb_id' => ['Field "kb_id" is required']]);
        }

        if (empty($body['query'])) {
            throw new ValidationException(['query' => ['Field "query" is required']]);
        }

        if (mb_strlen($body['query']) > 1000) {
            throw new ValidationException(['query' => ['Query exceeds maximum length of 1000 characters']]);
        }
    }

    /**
     * Validate ingest request body
     *
     * @param mixed $body
     * @throws ValidationException
     */
    private function validateIngestRequest($body): void
    {
        if (!is_array($body)) {
            throw new ValidationException(['body' => ['Request body must be a JSON object']]);
        }

        if (empty($body['doc_id'])) {
            throw new ValidationException(['doc_id' => ['Field "doc_id" is required']]);
        }

        if (empty($body['kb_id'])) {
            throw new ValidationException(['kb_id' => ['Field "kb_id" is required']]);
        }

        if (empty($body['file_path'])) {
            throw new ValidationException(['file_path' => ['Field "file_path" is required']]);
        }

        if (empty($body['mime_type'])) {
            throw new ValidationException(['mime_type' => ['Field "mime_type" is required']]);
        }

        $allowedMimeTypes = [
            'application/pdf',
            'text/plain',
            'application/vnd.openxmlformats-officedocument.wordprocessingml.document', // .docx
            'application/msword', // .doc
        ];

        if (!in_array($body['mime_type'], $allowedMimeTypes, true)) {
            throw new ValidationException(['mime_type' => ['MIME type not supported: ' . $body['mime_type']]]);
        }
    }

    /**
     * Validate compare request body
     *
     * @param mixed $body
     * @throws ValidationException
     */
    private function validateCompareRequest($body): void
    {
        if (!is_array($body)) {
            throw new ValidationException(['body' => ['Request body must be a JSON object']]);
        }

        if (empty($body['kb_id'])) {
            throw new ValidationException(['kb_id' => ['Field "kb_id" is required']]);
        }

        if (empty($body['doc_ids']) || !is_array($body['doc_ids'])) {
            throw new ValidationException(['doc_ids' => ['Field "doc_ids" must be a non-empty array']]);
        }

        if (count($body['doc_ids']) < 2) {
            throw new ValidationException(['doc_ids' => ['At least 2 documents are required for comparison']]);
        }

        if (count($body['doc_ids']) > 10) {
            throw new ValidationException(['doc_ids' => ['Maximum 10 documents allowed for comparison']]);
        }

        if (empty($body['comparison_aspects']) || !is_array($body['comparison_aspects'])) {
            throw new ValidationException(['comparison_aspects' => ['Field "comparison_aspects" must be a non-empty array']]);
        }
    }
}

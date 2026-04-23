<?php

declare(strict_types=1);

namespace ArchivioParlante\Controller;

use ArchivioParlante\Service\RustEngineProxy;
use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Log\LoggerInterface;

final class HealthController
{
    public function __construct(
        private LoggerInterface $logger,
        private RustEngineProxy $rustProxy
    ) {
    }

    public function health(Request $request, Response $response): Response
    {
        $this->logger->info('Health check requested');

        $health = [
            'status' => 'ok',
            'service' => 'php-gateway',
            'version' => '0.1.0',
            'timestamp' => time(),
        ];

        // Check Rust engine connectivity
        try {
            $rustHealth = $this->rustProxy->checkHealth();
            $health['rust_engine'] = $rustHealth['status'] === 'ok' ? 'connected' : 'error';
        } catch (\Exception $e) {
            $this->logger->warning('Rust engine health check failed', ['error' => $e->getMessage()]);
            $health['rust_engine'] = 'unreachable';
        }

        $response->getBody()->write(json_encode($health, JSON_THROW_ON_ERROR));

        return $response
            ->withHeader('Content-Type', 'application/json')
            ->withStatus(200);
    }
}

<?php

declare(strict_types=1);

namespace ArchivioParlante\Service;

use GuzzleHttp\Client;
use GuzzleHttp\Exception\GuzzleException;
use Psr\Log\LoggerInterface;

final class RustEngineProxy
{
    public function __construct(
        private Client $httpClient,
        private LoggerInterface $logger,
        private string $rustEngineUrl,
        private string $internalToken
    ) {
    }

    /**
     * Check Rust engine health
     *
     * @return array{status: string, service: string}
     * @throws \RuntimeException
     */
    public function checkHealth(): array
    {
        $url = rtrim($this->rustEngineUrl, '/') . '/health';

        $this->logger->debug('Checking Rust engine health', ['url' => $url]);

        try {
            $response = $this->httpClient->get($url, [
                'timeout' => 5.0,
            ]);

            if ($response->getStatusCode() !== 200) {
                throw new \RuntimeException(
                    sprintf('Rust engine returned status %d', $response->getStatusCode())
                );
            }

            $body = (string) $response->getBody();
            $data = json_decode($body, true, 512, JSON_THROW_ON_ERROR);

            return $data;
        } catch (GuzzleException $e) {
            $this->logger->error('Rust engine request failed', [
                'error' => $e->getMessage(),
                'url' => $url,
            ]);

            throw new \RuntimeException('Failed to connect to Rust engine: ' . $e->getMessage(), 0, $e);
        }
    }

    /**
     * Proxy request to Rust engine
     *
     * @param string $method HTTP method (GET, POST, etc.)
     * @param string $path Endpoint path (e.g., '/query')
     * @param array<string, mixed> $data Request payload
     * @return array<string, mixed> Response data
     * @throws \RuntimeException
     */
    public function proxyRequest(string $method, string $path, array $data = []): array
    {
        $url = rtrim($this->rustEngineUrl, '/') . '/' . ltrim($path, '/');

        $this->logger->info('Proxying request to Rust engine', [
            'method' => $method,
            'path' => $path,
        ]);

        try {
            $options = [
                'timeout' => 30.0,
                'headers' => [
                    'Content-Type' => 'application/json',
                ],
            ];

            // Add internal auth token
            if (!empty($this->internalToken)) {
                $options['headers']['X-Internal-Token'] = $this->internalToken;
            }

            // Add body for POST/PUT requests
            if (in_array(strtoupper($method), ['POST', 'PUT', 'PATCH']) && !empty($data)) {
                $options['json'] = $data;
            }

            $response = $this->httpClient->request($method, $url, $options);

            $statusCode = $response->getStatusCode();
            $body = (string) $response->getBody();

            // Handle non-2xx responses
            if ($statusCode < 200 || $statusCode >= 300) {
                $this->logger->warning('Rust engine returned error', [
                    'status' => $statusCode,
                    'body' => $body,
                ]);

                // Try to parse error message
                $errorData = json_decode($body, true);
                $errorMessage = $errorData['error'] ?? 'Unknown error';

                throw new \RuntimeException(
                    sprintf('Rust engine error (%d): %s', $statusCode, $errorMessage),
                    $statusCode
                );
            }

            return json_decode($body, true, 512, JSON_THROW_ON_ERROR);
        } catch (GuzzleException $e) {
            $this->logger->error('Proxy request failed', [
                'error' => $e->getMessage(),
                'method' => $method,
                'path' => $path,
            ]);

            throw new \RuntimeException('Proxy request failed: ' . $e->getMessage(), 0, $e);
        }
    }
}

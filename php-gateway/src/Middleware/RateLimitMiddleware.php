<?php

declare(strict_types=1);

namespace ArchivioParlante\Middleware;

use ArchivioParlante\Service\AuditLogger;
use ArchivioParlante\Service\RedisSessionManager;
use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface as RequestHandler;
use Psr\Log\LoggerInterface;

/**
 * Configurable Rate Limiting Middleware (PSR-15)
 *
 * Protects endpoints from brute-force and abuse.
 * Rate limits are configurable per endpoint via constructor parameters.
 *
 * Usage:
 * Create factory instances in container.php with different limits:
 * - Login: 5 attempts / 15 min
 * - Register: 3 attempts / 1 hour
 * - Refresh: 20 attempts / 15 min
 *
 * Redis key pattern: ratelimit:{endpoint}:{ip}
 * All violations are logged to ap_audit_logs table.
 */
final class RateLimitMiddleware implements MiddlewareInterface
{
    public function __construct(
        private RedisSessionManager $sessionManager,
        private AuditLogger $auditLogger,
        private LoggerInterface $logger,
        private int $maxAttempts,
        private int $windowSeconds,
        private string $endpointName
    ) {
    }

    /**
     * Process request and check rate limit
     *
     * @param Request $request PSR-7 request
     * @param RequestHandler $handler Request handler
     * @return Response PSR-7 response
     */
    public function process(Request $request, RequestHandler $handler): Response
    {
        // Get client IP address
        $serverParams = $request->getServerParams();
        $ipAddress = $serverParams['REMOTE_ADDR'] ?? '127.0.0.1';
        $userAgent = $request->getHeaderLine('User-Agent') ?: 'Unknown';
        $requestPath = $request->getUri()->getPath();
        $requestMethod = $request->getMethod();

        // Build endpoint-specific Redis key
        $rateLimitKey = sprintf('ratelimit:%s:%s', $this->endpointName, $ipAddress);

        // Check current attempt count
        $attempts = $this->sessionManager->getRateLimitAttempts($rateLimitKey);

        if ($attempts >= $this->maxAttempts) {
            $this->logger->warning('Rate limit exceeded', [
                'endpoint' => $this->endpointName,
                'ip' => $ipAddress,
                'attempts' => $attempts,
                'max_attempts' => $this->maxAttempts,
            ]);

            // Log violation to audit log
            $this->auditLogger->logRateLimitViolation(
                ipAddress: $ipAddress,
                userAgent: $userAgent,
                requestPath: $requestPath,
                requestMethod: $requestMethod,
                attempts: $attempts
            );

            return $this->createTooManyRequestsResponse();
        }

        // Allow request to proceed
        $this->logger->debug('Rate limit check passed', [
            'endpoint' => $this->endpointName,
            'ip' => $ipAddress,
            'attempts' => $attempts,
            'remaining' => $this->maxAttempts - $attempts,
        ]);

        return $handler->handle($request);
    }

    /**
     * Create 429 Too Many Requests JSON response
     *
     * @return Response 429 response
     */
    private function createTooManyRequestsResponse(): Response
    {
        $windowMinutes = (int) ($this->windowSeconds / 60);
        $windowHours = $this->windowSeconds >= 3600 ? (int) ($this->windowSeconds / 3600) : null;

        $timeDescription = $windowHours !== null
            ? sprintf('%d hour%s', $windowHours, $windowHours > 1 ? 's' : '')
            : sprintf('%d minute%s', $windowMinutes, $windowMinutes > 1 ? 's' : '');

        $responseBody = json_encode([
            'error' => 'Too many requests',
            'message' => sprintf(
                'Maximum %d requests exceeded for %s. Please try again in %s.',
                $this->maxAttempts,
                $this->endpointName,
                $timeDescription
            ),
            'retry_after' => $this->windowSeconds,
        ], JSON_THROW_ON_ERROR);

        // Create new PSR-7 response
        $response = new \Slim\Psr7\Response();
        $response->getBody()->write($responseBody);

        return $response
            ->withHeader('Content-Type', 'application/json')
            ->withHeader('Retry-After', (string) $this->windowSeconds)
            ->withStatus(429);
    }
}

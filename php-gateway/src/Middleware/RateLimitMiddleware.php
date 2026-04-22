<?php

declare(strict_types=1);

namespace ArchivioParlante\Middleware;

use ArchivioParlante\Service\RedisSessionManager;
use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface as RequestHandler;
use Psr\Log\LoggerInterface;

/**
 * Rate Limiting Middleware (PSR-15)
 *
 * Protects endpoints from brute-force attacks.
 * Limits login attempts per IP address to 5 per 15 minutes.
 *
 * Usage:
 * - Apply to /api/auth/login route via ->add(RateLimitMiddleware::class)
 *
 * Rate limit:
 * - 5 attempts per 15 minutes (900 seconds)
 * - Tracked by client IP address in Redis
 * - Returns 429 Too Many Requests when limit exceeded
 */
final class RateLimitMiddleware implements MiddlewareInterface
{
    private const MAX_ATTEMPTS = 5;
    private const WINDOW_SECONDS = 900; // 15 minutes

    public function __construct(
        private RedisSessionManager $sessionManager,
        private LoggerInterface $logger
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

        // Check current attempt count
        $attempts = $this->sessionManager->getLoginAttempts($ipAddress);

        if ($attempts >= self::MAX_ATTEMPTS) {
            $this->logger->warning('Rate limit exceeded', [
                'ip' => $ipAddress,
                'attempts' => $attempts,
                'max_attempts' => self::MAX_ATTEMPTS,
            ]);

            return $this->createTooManyRequestsResponse();
        }

        // Allow request to proceed
        $this->logger->debug('Rate limit check passed', [
            'ip' => $ipAddress,
            'attempts' => $attempts,
            'remaining' => self::MAX_ATTEMPTS - $attempts,
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
        $responseBody = json_encode([
            'error' => 'Too many requests',
            'message' => sprintf(
                'Maximum %d login attempts exceeded. Please try again in %d minutes.',
                self::MAX_ATTEMPTS,
                (int) (self::WINDOW_SECONDS / 60)
            ),
            'retry_after' => self::WINDOW_SECONDS,
        ], JSON_THROW_ON_ERROR);

        // Create new PSR-7 response
        $response = new \Slim\Psr7\Response();
        $response->getBody()->write($responseBody);

        return $response
            ->withHeader('Content-Type', 'application/json')
            ->withHeader('Retry-After', (string) self::WINDOW_SECONDS)
            ->withStatus(429);
    }
}

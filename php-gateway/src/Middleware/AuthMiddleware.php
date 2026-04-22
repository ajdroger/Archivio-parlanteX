<?php

declare(strict_types=1);

namespace ArchivioParlante\Middleware;

use ArchivioParlante\Exception\AuthenticationException;
use ArchivioParlante\Service\JwtService;
use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface as RequestHandler;
use Psr\Log\LoggerInterface;

/**
 * JWT Authentication Middleware (PSR-15)
 *
 * Validates JWT access token from Authorization header.
 * Adds user data to request attributes on successful validation.
 * Returns 401 JSON response on authentication failure.
 *
 * Usage:
 * - Apply to protected routes via ->add(AuthMiddleware::class)
 * - Do NOT apply globally (login/register must be public)
 *
 * Request attribute added:
 * - 'user' => ['sub' => userId, 'email' => email, 'role' => role, 'iat' => timestamp, 'exp' => timestamp]
 */
final class AuthMiddleware implements MiddlewareInterface
{
    public function __construct(
        private JwtService $jwtService,
        private LoggerInterface $logger
    ) {
    }

    /**
     * Process request and validate JWT token
     *
     * @param Request $request PSR-7 request
     * @param RequestHandler $handler Request handler
     * @return Response PSR-7 response
     */
    public function process(Request $request, RequestHandler $handler): Response
    {
        try {
            // Extract Authorization header
            $authHeader = $request->getHeaderLine('Authorization');

            if ($authHeader === '') {
                $this->logger->debug('Missing Authorization header');
                throw AuthenticationException::missingAuthHeader();
            }

            // Extract token from "Bearer <token>" format
            $token = $this->jwtService->extractTokenFromHeader($authHeader);

            if ($token === null) {
                $this->logger->debug('Invalid Authorization header format');
                throw AuthenticationException::invalidToken('Invalid Authorization header format');
            }

            // Validate and decode token
            $payload = $this->jwtService->validateAccessToken($token);

            // Add user data to request attributes for downstream controllers
            $request = $request->withAttribute('user', $payload);

            $this->logger->debug('Request authenticated', [
                'user_id' => $payload['sub'],
                'email' => $payload['email'],
                'role' => $payload['role'],
            ]);

            // Continue to next middleware/controller
            return $handler->handle($request);
        } catch (AuthenticationException $e) {
            // Return 401 JSON response
            return $this->createUnauthorizedResponse($request, $e->getMessage());
        } catch (\Exception $e) {
            // Log unexpected errors
            $this->logger->error('Authentication middleware error', [
                'error' => $e->getMessage(),
                'trace' => $e->getTraceAsString(),
            ]);

            return $this->createUnauthorizedResponse($request, 'Authentication failed');
        }
    }

    /**
     * Create 401 Unauthorized JSON response
     *
     * @param Request $request Original request
     * @param string $message Error message
     * @return Response 401 response
     */
    private function createUnauthorizedResponse(Request $request, string $message): Response
    {
        $responseBody = json_encode([
            'error' => 'Unauthorized',
            'message' => $message,
        ], JSON_THROW_ON_ERROR);

        // Create new PSR-7 response
        $response = new \Slim\Psr7\Response();
        $response->getBody()->write($responseBody);

        return $response
            ->withHeader('Content-Type', 'application/json')
            ->withStatus(401);
    }
}

<?php
declare(strict_types=1);

namespace ArchivioParlante\Middleware;

use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Server\MiddlewareInterface;
use Psr\Http\Server\RequestHandlerInterface;
use Slim\Psr7\Response;

class CsrfMiddleware implements MiddlewareInterface
{
    private const TOKEN_LENGTH = 32;

    public function process(ServerRequestInterface $request, RequestHandlerInterface $handler): ResponseInterface
    {
        // Skip CSRF for safe methods
        $method = $request->getMethod();
        if (in_array($method, ['GET', 'HEAD', 'OPTIONS'])) {
            return $handler->handle($request);
        }

        // Validate CSRF token for state-changing operations
        $headerToken = $request->getHeaderLine('X-CSRF-Token');
        $cookies = $request->getCookieParams();
        $cookieToken = $cookies['csrf_token'] ?? null;

        if ($headerToken === '' || $cookieToken === null || !hash_equals($cookieToken, $headerToken)) {
            return $this->forbidden('CSRF token validation failed');
        }

        $response = $handler->handle($request);

        // Rotate CSRF token after use
        return $this->setNewCsrfToken($response);
    }

    private function setNewCsrfToken(ResponseInterface $response): ResponseInterface
    {
        $token = bin2hex(random_bytes(self::TOKEN_LENGTH));

        return $response->withHeader('Set-Cookie', sprintf(
            'csrf_token=%s; Path=/; HttpOnly; SameSite=Strict; Secure',
            $token
        ));
    }

    private function forbidden(string $message): ResponseInterface
    {
        $response = new Response();
        $response->getBody()->write(json_encode(['error' => $message]));
        return $response
            ->withStatus(403)
            ->withHeader('Content-Type', 'application/json');
    }
}

<?php

declare(strict_types=1);

use ArchivioParlante\Controller\AuthController;
use ArchivioParlante\Controller\HealthController;
use ArchivioParlante\Controller\ProxyController;
use ArchivioParlante\Middleware\AuthMiddleware;
use Slim\App;

return function (App $app): void {
    // Health check
    $app->get('/health', HealthController::class . ':health');

    // === Fase 3.2+3.3: Authentication Routes ===

    // Public routes (no auth required, with configurable rate limiting)
    $app->post('/api/auth/register', AuthController::class . ':register')
        ->add('RateLimitMiddleware.register'); // 3 attempts / 1 hour
    $app->post('/api/auth/login', AuthController::class . ':login')
        ->add('RateLimitMiddleware.login'); // 5 attempts / 15 min
    $app->post('/api/auth/refresh', AuthController::class . ':refresh')
        ->add('RateLimitMiddleware.refresh'); // 20 attempts / 15 min

    // Protected routes (require JWT access token)
    $app->post('/api/auth/logout', AuthController::class . ':logout')
        ->add(AuthMiddleware::class);
    $app->get('/api/auth/me', AuthController::class . ':me')
        ->add(AuthMiddleware::class);

    // === Fase 3.4: Proxy routes to Rust engine (protected with AuthMiddleware) ===

    $app->post('/api/query', ProxyController::class . ':query')
        ->add(AuthMiddleware::class);

    $app->post('/api/ingest', ProxyController::class . ':ingest')
        ->add(AuthMiddleware::class);

    $app->post('/api/compare', ProxyController::class . ':compare')
        ->add(AuthMiddleware::class);
};

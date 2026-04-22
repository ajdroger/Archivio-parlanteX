<?php

declare(strict_types=1);

use ArchivioParlante\Controller\AuthController;
use ArchivioParlante\Controller\HealthController;
use ArchivioParlante\Middleware\AuthMiddleware;
use ArchivioParlante\Middleware\RateLimitMiddleware;
use Slim\App;

return function (App $app): void {
    // Health check
    $app->get('/health', HealthController::class . ':health');

    // === Fase 3.2: Authentication Routes ===

    // Public routes (no auth required)
    $app->post('/api/auth/register', AuthController::class . ':register');
    $app->post('/api/auth/login', AuthController::class . ':login')
        ->add(RateLimitMiddleware::class); // Rate limiting: 5 attempts per 15 min
    $app->post('/api/auth/refresh', AuthController::class . ':refresh');

    // Protected routes (require JWT access token)
    $app->post('/api/auth/logout', AuthController::class . ':logout')
        ->add(AuthMiddleware::class);
    $app->get('/api/auth/me', AuthController::class . ':me')
        ->add(AuthMiddleware::class);

    // TODO Fase 3.3: Proxy routes to Rust engine (will be protected with AuthMiddleware)
    // POST /api/query
    // POST /api/ingest
    // POST /api/compare
};

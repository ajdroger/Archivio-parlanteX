<?php

declare(strict_types=1);

use ArchivioParlante\Controller\HealthController;
use Slim\App;

return function (App $app): void {
    // Health check
    $app->get('/health', HealthController::class . ':health');

    // TODO Fase 3.2: Auth routes
    // POST /api/auth/login
    // POST /api/auth/logout
    // GET /api/auth/me

    // TODO Fase 3.3: Proxy routes to Rust engine
    // POST /api/query
    // POST /api/ingest
    // POST /api/compare
};

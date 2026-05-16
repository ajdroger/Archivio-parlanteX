<?php

declare(strict_types=1);

use ArchivioParlante\Controller\HealthController;
use ArchivioParlante\Controller\AuthController;
use ArchivioParlante\Controller\WorkspaceController;
use ArchivioParlante\Controller\ProxyController;
use ArchivioParlante\Middleware\AuthMiddleware;
use Slim\App;

return function (App $app): void {
    // Health check
    $app->get('/health', HealthController::class . ':health');

    // Auth routes (public, no middleware)
    $app->group('/api/auth', function ($group) {
        $group->post('/register', AuthController::class . ':register');
        $group->post('/login', AuthController::class . ':login');
        $group->post('/refresh', AuthController::class . ':refresh');
        $group->post('/logout', AuthController::class . ':logout');
        $group->get('/me', AuthController::class . ':me')->add(AuthMiddleware::class);
    });

    // Proxy routes to Rust engine (protected by auth)
    $app->group('/api', function ($group) {
        $group->post('/query', ProxyController::class . ':query');
        $group->post('/ingest', ProxyController::class . ':ingest');
        $group->post('/compare', ProxyController::class . ':compare');
    })->add(AuthMiddleware::class);

    // Fase 6.3: Workspace Management Routes (protected by auth)
    $app->group('/api/workspaces', function ($group) {
        // Workspace CRUD
        $group->get('', WorkspaceController::class . ':listWorkspaces');
        $group->post('', WorkspaceController::class . ':createWorkspace');
        $group->get('/{id}', WorkspaceController::class . ':getWorkspace');
        $group->delete('/{id}', WorkspaceController::class . ':deleteWorkspace');

        // Workspace Member Management
        $group->get('/{id}/members', WorkspaceController::class . ':listMembers');
        $group->post('/{id}/members', WorkspaceController::class . ':addMember');
        $group->delete('/{id}/members/{userId}', WorkspaceController::class . ':removeMember');
        $group->patch('/{id}/members/{userId}', WorkspaceController::class . ':updateRole');
    })->add(AuthMiddleware::class);
};

<?php

declare(strict_types=1);

use ArchivioParlante\Controller\HealthController;
use ArchivioParlante\Controller\WorkspaceController;
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

    // Fase 6.3: Workspace Management Routes (require auth when middleware available)
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
    });
    // TODO: Add auth middleware to workspace routes when AuthMiddleware is implemented
};

<?php

declare(strict_types=1);

namespace ArchivioParlante\Controller;

use ArchivioParlante\Service\WorkspaceService;
use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Slim\Exception\HttpBadRequestException;
use Slim\Exception\HttpForbiddenException;
use Slim\Exception\HttpNotFoundException;

/**
 * Workspace Controller - API endpoints for multi-tenant workspace management
 *
 * Fase 6.3.3: REST API for workspaces, members, and permissions
 */
class WorkspaceController
{
    private WorkspaceService $workspaceService;

    public function __construct(WorkspaceService $workspaceService)
    {
        $this->workspaceService = $workspaceService;
    }

    /**
     * GET /api/workspaces
     * List all workspaces for authenticated user
     */
    public function listWorkspaces(Request $request, Response $response): Response
    {
        // Get authenticated user ID from request attributes (set by AuthMiddleware)
        $userId = $request->getAttribute('user_id');

        if (!$userId) {
            throw new HttpForbiddenException($request, 'Authentication required');
        }

        $workspaces = $this->workspaceService->listWorkspacesForUser((int)$userId);

        $response->getBody()->write(json_encode([
            'workspaces' => $workspaces,
            'count' => count($workspaces),
        ]));

        return $response->withHeader('Content-Type', 'application/json');
    }

    /**
     * GET /api/workspaces/{id}
     * Get single workspace details
     */
    public function getWorkspace(Request $request, Response $response, array $args): Response
    {
        $userId = $request->getAttribute('user_id');
        $workspaceId = $args['id'] ?? null;

        if (!$workspaceId) {
            throw new HttpBadRequestException($request, 'Workspace ID required');
        }

        // Check user is member of workspace
        $role = $this->workspaceService->getUserRole($workspaceId, (int)$userId);
        if (!$role) {
            throw new HttpForbiddenException($request, 'Access denied to this workspace');
        }

        $workspace = $this->workspaceService->getWorkspace($workspaceId);
        if (!$workspace) {
            throw new HttpNotFoundException($request, 'Workspace not found');
        }

        // Add user's role to response
        $workspace['your_role'] = $role;

        $response->getBody()->write(json_encode($workspace));
        return $response->withHeader('Content-Type', 'application/json');
    }

    /**
     * POST /api/workspaces
     * Create new workspace
     */
    public function createWorkspace(Request $request, Response $response): Response
    {
        $userId = $request->getAttribute('user_id');
        $data = $request->getParsedBody();

        $name = $data['name'] ?? null;
        if (!$name || trim($name) === '') {
            throw new HttpBadRequestException($request, 'Workspace name is required');
        }

        try {
            $workspace = $this->workspaceService->createWorkspace(
                trim($name),
                (int)$userId
            );

            $response->getBody()->write(json_encode($workspace));
            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(201);
        } catch (\RuntimeException $e) {
            throw new HttpBadRequestException($request, $e->getMessage());
        }
    }

    /**
     * DELETE /api/workspaces/{id}
     * Delete workspace (soft delete, admin only)
     */
    public function deleteWorkspace(Request $request, Response $response, array $args): Response
    {
        $userId = $request->getAttribute('user_id');
        $workspaceId = $args['id'] ?? null;

        if (!$workspaceId) {
            throw new HttpBadRequestException($request, 'Workspace ID required');
        }

        // Check user is admin of workspace
        if (!$this->workspaceService->isAdmin($workspaceId, (int)$userId)) {
            throw new HttpForbiddenException($request, 'Only workspace admins can delete workspaces');
        }

        $success = $this->workspaceService->deleteWorkspace($workspaceId);

        $response->getBody()->write(json_encode([
            'success' => $success,
            'message' => $success ? 'Workspace deleted' : 'Workspace not found',
        ]));

        return $response->withHeader('Content-Type', 'application/json');
    }

    /**
     * GET /api/workspaces/{id}/members
     * List workspace members
     */
    public function listMembers(Request $request, Response $response, array $args): Response
    {
        $userId = $request->getAttribute('user_id');
        $workspaceId = $args['id'] ?? null;

        if (!$workspaceId) {
            throw new HttpBadRequestException($request, 'Workspace ID required');
        }

        // Check user is member of workspace
        $role = $this->workspaceService->getUserRole($workspaceId, (int)$userId);
        if (!$role) {
            throw new HttpForbiddenException($request, 'Access denied to this workspace');
        }

        $members = $this->workspaceService->listMembers($workspaceId);

        $response->getBody()->write(json_encode([
            'members' => $members,
            'count' => count($members),
        ]));

        return $response->withHeader('Content-Type', 'application/json');
    }

    /**
     * POST /api/workspaces/{id}/members
     * Add member to workspace (admin only)
     */
    public function addMember(Request $request, Response $response, array $args): Response
    {
        $userId = $request->getAttribute('user_id');
        $workspaceId = $args['id'] ?? null;
        $data = $request->getParsedBody();

        if (!$workspaceId) {
            throw new HttpBadRequestException($request, 'Workspace ID required');
        }

        // Check user is admin of workspace
        if (!$this->workspaceService->isAdmin($workspaceId, (int)$userId)) {
            throw new HttpForbiddenException($request, 'Only workspace admins can add members');
        }

        $targetUserId = $data['user_id'] ?? null;
        $role = $data['role'] ?? 'member';

        if (!$targetUserId) {
            throw new HttpBadRequestException($request, 'User ID is required');
        }

        // Validate role
        if (!in_array($role, ['admin', 'member', 'viewer'], true)) {
            throw new HttpBadRequestException($request, 'Invalid role. Must be: admin, member, or viewer');
        }

        try {
            $success = $this->workspaceService->addMember(
                $workspaceId,
                (int)$targetUserId,
                $role,
                (int)$userId
            );

            $response->getBody()->write(json_encode([
                'success' => $success,
                'message' => 'Member added to workspace',
            ]));

            return $response
                ->withHeader('Content-Type', 'application/json')
                ->withStatus(201);
        } catch (\RuntimeException $e) {
            throw new HttpBadRequestException($request, $e->getMessage());
        }
    }

    /**
     * DELETE /api/workspaces/{id}/members/{userId}
     * Remove member from workspace (admin only)
     */
    public function removeMember(Request $request, Response $response, array $args): Response
    {
        $userId = $request->getAttribute('user_id');
        $workspaceId = $args['id'] ?? null;
        $targetUserId = $args['userId'] ?? null;

        if (!$workspaceId || !$targetUserId) {
            throw new HttpBadRequestException($request, 'Workspace ID and User ID required');
        }

        // Check user is admin of workspace
        if (!$this->workspaceService->isAdmin($workspaceId, (int)$userId)) {
            throw new HttpForbiddenException($request, 'Only workspace admins can remove members');
        }

        // Prevent removing workspace owner
        $workspace = $this->workspaceService->getWorkspace($workspaceId);
        if ($workspace && $workspace['owner_user_id'] == $targetUserId) {
            throw new HttpBadRequestException($request, 'Cannot remove workspace owner');
        }

        $success = $this->workspaceService->removeMember($workspaceId, (int)$targetUserId);

        $response->getBody()->write(json_encode([
            'success' => $success,
            'message' => $success ? 'Member removed' : 'Member not found',
        ]));

        return $response->withHeader('Content-Type', 'application/json');
    }

    /**
     * PATCH /api/workspaces/{id}/members/{userId}
     * Update member role (admin only)
     */
    public function updateMemberRole(Request $request, Response $response, array $args): Response
    {
        $userId = $request->getAttribute('user_id');
        $workspaceId = $args['id'] ?? null;
        $targetUserId = $args['userId'] ?? null;
        $data = $request->getParsedBody();

        if (!$workspaceId || !$targetUserId) {
            throw new HttpBadRequestException($request, 'Workspace ID and User ID required');
        }

        // Check user is admin of workspace
        if (!$this->workspaceService->isAdmin($workspaceId, (int)$userId)) {
            throw new HttpForbiddenException($request, 'Only workspace admins can update member roles');
        }

        $newRole = $data['role'] ?? null;
        if (!$newRole || !in_array($newRole, ['admin', 'member', 'viewer'], true)) {
            throw new HttpBadRequestException($request, 'Valid role required (admin, member, or viewer)');
        }

        $success = $this->workspaceService->updateMemberRole(
            $workspaceId,
            (int)$targetUserId,
            $newRole
        );

        $response->getBody()->write(json_encode([
            'success' => $success,
            'message' => $success ? 'Member role updated' : 'Member not found',
        ]));

        return $response->withHeader('Content-Type', 'application/json');
    }
}

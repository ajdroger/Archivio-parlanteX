<?php

declare(strict_types=1);

namespace ArchivioParlante\Service;

use PDO;
use PDOException;
use RuntimeException;

/**
 * Workspace Service - Multi-tenant workspace management
 *
 * Fase 6.3.3: CRUD operations for workspaces, members, and permissions
 */
class WorkspaceService
{
    private PDO $db;

    public function __construct(PDO $db)
    {
        $this->db = $db;
    }

    /**
     * Create new workspace
     *
     * @param string $name Workspace name
     * @param int $ownerUserId User ID of workspace owner
     * @return array Created workspace data
     * @throws RuntimeException If creation fails
     */
    public function createWorkspace(string $name, int $ownerUserId): array
    {
        try {
            $workspaceId = $this->generateUuid();

            $stmt = $this->db->prepare(
                'INSERT INTO ap_workspaces (id, name, owner_user_id, created_at)
                VALUES (:id, :name, :owner_user_id, NOW())'
            );

            $stmt->execute([
                'id' => $workspaceId,
                'name' => $name,
                'owner_user_id' => $ownerUserId,
            ]);

            // Auto-add owner as admin member
            $this->addMember($workspaceId, $ownerUserId, 'admin', $ownerUserId);

            return $this->getWorkspace($workspaceId);
        } catch (PDOException $e) {
            throw new RuntimeException('Failed to create workspace: ' . $e->getMessage());
        }
    }

    /**
     * Get workspace by ID
     *
     * @param string $workspaceId Workspace UUID
     * @return array|null Workspace data or null if not found
     */
    public function getWorkspace(string $workspaceId): ?array
    {
        $stmt = $this->db->prepare(
            'SELECT
                w.id,
                w.name,
                w.owner_user_id,
                w.created_at,
                w.updated_at,
                u.email as owner_email,
                (SELECT COUNT(*) FROM ap_workspace_members WHERE workspace_id = w.id) as member_count,
                (SELECT COUNT(*) FROM ap_knowledge_bases WHERE workspace_id = w.id AND deleted_at IS NULL) as kb_count
            FROM ap_workspaces w
            INNER JOIN ap_users u ON w.owner_user_id = u.id
            WHERE w.id = :id AND w.deleted_at IS NULL'
        );

        $stmt->execute(['id' => $workspaceId]);
        $result = $stmt->fetch(PDO::FETCH_ASSOC);

        return $result ?: null;
    }

    /**
     * List workspaces for a user
     *
     * @param int $userId User ID
     * @return array Array of workspaces user has access to
     */
    public function listWorkspacesForUser(int $userId): array
    {
        $stmt = $this->db->prepare(
            'SELECT
                w.id,
                w.name,
                w.owner_user_id,
                w.created_at,
                m.role as user_role,
                (SELECT COUNT(*) FROM ap_workspace_members WHERE workspace_id = w.id) as member_count,
                (SELECT COUNT(*) FROM ap_knowledge_bases WHERE workspace_id = w.id AND deleted_at IS NULL) as kb_count
            FROM ap_workspaces w
            INNER JOIN ap_workspace_members m ON w.id = m.workspace_id
            WHERE m.user_id = :user_id AND w.deleted_at IS NULL
            ORDER BY w.name ASC'
        );

        $stmt->execute(['user_id' => $userId]);
        return $stmt->fetchAll(PDO::FETCH_ASSOC);
    }

    /**
     * Add member to workspace
     *
     * @param string $workspaceId Workspace UUID
     * @param int $userId User ID to add
     * @param string $role Member role (admin, member, viewer)
     * @param int $invitedBy User ID who is adding the member
     * @return bool Success
     * @throws RuntimeException If add fails
     */
    public function addMember(string $workspaceId, int $userId, string $role, int $invitedBy): bool
    {
        try {
            $stmt = $this->db->prepare(
                'INSERT INTO ap_workspace_members (workspace_id, user_id, role, joined_at, invited_by)
                VALUES (:workspace_id, :user_id, :role, NOW(), :invited_by)
                ON DUPLICATE KEY UPDATE role = :role'
            );

            $stmt->execute([
                'workspace_id' => $workspaceId,
                'user_id' => $userId,
                'role' => $role,
                'invited_by' => $invitedBy,
            ]);

            return true;
        } catch (PDOException $e) {
            throw new RuntimeException('Failed to add member: ' . $e->getMessage());
        }
    }

    /**
     * Remove member from workspace
     *
     * @param string $workspaceId Workspace UUID
     * @param int $userId User ID to remove
     * @return bool Success
     */
    public function removeMember(string $workspaceId, int $userId): bool
    {
        $stmt = $this->db->prepare(
            'DELETE FROM ap_workspace_members
            WHERE workspace_id = :workspace_id AND user_id = :user_id'
        );

        $stmt->execute([
            'workspace_id' => $workspaceId,
            'user_id' => $userId,
        ]);

        return $stmt->rowCount() > 0;
    }

    /**
     * Update member role
     *
     * @param string $workspaceId Workspace UUID
     * @param int $userId User ID
     * @param string $newRole New role (admin, member, viewer)
     * @return bool Success
     */
    public function updateMemberRole(string $workspaceId, int $userId, string $newRole): bool
    {
        $stmt = $this->db->prepare(
            'UPDATE ap_workspace_members
            SET role = :role
            WHERE workspace_id = :workspace_id AND user_id = :user_id'
        );

        $stmt->execute([
            'workspace_id' => $workspaceId,
            'user_id' => $userId,
            'role' => $newRole,
        ]);

        return $stmt->rowCount() > 0;
    }

    /**
     * List members of a workspace
     *
     * @param string $workspaceId Workspace UUID
     * @return array Array of workspace members with user details
     */
    public function listMembers(string $workspaceId): array
    {
        $stmt = $this->db->prepare(
            'SELECT
                m.user_id,
                m.role,
                m.joined_at,
                u.email,
                u.full_name,
                inviter.email as invited_by_email
            FROM ap_workspace_members m
            INNER JOIN ap_users u ON m.user_id = u.id
            LEFT JOIN ap_users inviter ON m.invited_by = inviter.id
            WHERE m.workspace_id = :workspace_id
            ORDER BY m.joined_at ASC'
        );

        $stmt->execute(['workspace_id' => $workspaceId]);
        return $stmt->fetchAll(PDO::FETCH_ASSOC);
    }

    /**
     * Check if user is member of workspace
     *
     * @param string $workspaceId Workspace UUID
     * @param int $userId User ID
     * @return string|null Member role if member, null if not
     */
    public function getUserRole(string $workspaceId, int $userId): ?string
    {
        $stmt = $this->db->prepare(
            'SELECT role FROM ap_workspace_members
            WHERE workspace_id = :workspace_id AND user_id = :user_id'
        );

        $stmt->execute([
            'workspace_id' => $workspaceId,
            'user_id' => $userId,
        ]);

        $result = $stmt->fetch(PDO::FETCH_ASSOC);
        return $result ? $result['role'] : null;
    }

    /**
     * Check if user is admin of workspace
     *
     * @param string $workspaceId Workspace UUID
     * @param int $userId User ID
     * @return bool True if admin, false otherwise
     */
    public function isAdmin(string $workspaceId, int $userId): bool
    {
        return $this->getUserRole($workspaceId, $userId) === 'admin';
    }

    /**
     * Delete workspace (soft delete)
     *
     * @param string $workspaceId Workspace UUID
     * @return bool Success
     */
    public function deleteWorkspace(string $workspaceId): bool
    {
        $stmt = $this->db->prepare(
            'UPDATE ap_workspaces
            SET deleted_at = NOW()
            WHERE id = :id'
        );

        $stmt->execute(['id' => $workspaceId]);
        return $stmt->rowCount() > 0;
    }

    /**
     * Generate UUID v4
     *
     * @return string UUID
     */
    private function generateUuid(): string
    {
        $data = random_bytes(16);
        $data[6] = chr(ord($data[6]) & 0x0f | 0x40); // Version 4
        $data[8] = chr(ord($data[8]) & 0x3f | 0x80); // Variant

        return vsprintf('%s%s-%s-%s-%s-%s%s%s', str_split(bin2hex($data), 4));
    }
}

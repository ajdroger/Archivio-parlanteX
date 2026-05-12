-- ============================================================================
-- Archivio Parlante — Migration 010: Multi-Tenant Workspaces & Permissions
-- ============================================================================
-- Fase 6.3: Workspace-level isolation + KB sharing permissions
-- Author: Claude Sonnet 4.5
-- Date: 2026-05-07

-- === 1. WORKSPACES TABLE ===
-- Team/organization boundary for KB grouping
CREATE TABLE IF NOT EXISTS ap_workspaces (
    id CHAR(36) PRIMARY KEY COMMENT 'UUID workspace identifier',
    name VARCHAR(255) NOT NULL COMMENT 'Workspace display name (e.g., "Legal Team")',
    owner_user_id BIGINT UNSIGNED NOT NULL COMMENT 'User who created the workspace',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at DATETIME NULL COMMENT 'Soft delete timestamp',

    FOREIGN KEY (owner_user_id) REFERENCES ap_users(id) ON DELETE CASCADE,
    INDEX idx_workspace_owner (owner_user_id),
    INDEX idx_workspace_deleted (deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Workspaces for multi-tenant KB isolation';

-- === 2. WORKSPACE MEMBERS TABLE ===
-- Users belonging to workspaces with specific roles
CREATE TABLE IF NOT EXISTS ap_workspace_members (
    workspace_id CHAR(36) NOT NULL,
    user_id BIGINT UNSIGNED NOT NULL,
    role ENUM('admin', 'member', 'viewer') NOT NULL DEFAULT 'member' COMMENT 'admin=full control, member=read+write, viewer=read-only',
    joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    invited_by BIGINT UNSIGNED NULL COMMENT 'User who invited this member',

    PRIMARY KEY (workspace_id, user_id),
    FOREIGN KEY (workspace_id) REFERENCES ap_workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES ap_users(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES ap_users(id) ON DELETE SET NULL,

    INDEX idx_member_user (user_id),
    INDEX idx_member_role (role)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Workspace membership with role-based permissions';

-- === 3. KB PERMISSIONS TABLE ===
-- Granular access control for knowledge bases
CREATE TABLE IF NOT EXISTS ap_kb_permissions (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    kb_id CHAR(36) NOT NULL COMMENT 'Knowledge base identifier',
    user_id BIGINT UNSIGNED NULL COMMENT 'Specific user (if user-level permission)',
    workspace_id CHAR(36) NULL COMMENT 'Entire workspace (if workspace-level permission)',
    permission ENUM('read', 'write', 'admin') NOT NULL COMMENT 'read=query only, write=query+ingest, admin=full control',
    granted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    granted_by BIGINT UNSIGNED NULL COMMENT 'User who granted this permission',

    FOREIGN KEY (kb_id) REFERENCES ap_knowledge_bases(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES ap_users(id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id) REFERENCES ap_workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (granted_by) REFERENCES ap_users(id) ON DELETE SET NULL,

    -- Constraint: either user_id OR workspace_id must be set (not both, not neither)
    CHECK (
        (user_id IS NOT NULL AND workspace_id IS NULL) OR
        (user_id IS NULL AND workspace_id IS NOT NULL)
    ),

    -- Unique: one permission per (kb, user) or (kb, workspace)
    UNIQUE KEY unique_kb_user (kb_id, user_id),
    UNIQUE KEY unique_kb_workspace (kb_id, workspace_id),

    INDEX idx_permission_kb (kb_id),
    INDEX idx_permission_user (user_id),
    INDEX idx_permission_workspace (workspace_id),
    INDEX idx_permission_type (permission)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Granular KB access permissions (user-level or workspace-level)';

-- === 4. ADD WORKSPACE FOREIGN KEY TO KNOWLEDGE BASES ===
-- Link KB to parent workspace (optional: KB can exist without workspace for backward compat)
ALTER TABLE ap_knowledge_bases
ADD COLUMN workspace_id CHAR(36) NULL COMMENT 'Parent workspace (NULL = personal KB)' AFTER id,
ADD FOREIGN KEY fk_kb_workspace (workspace_id) REFERENCES ap_workspaces(id) ON DELETE SET NULL,
ADD INDEX idx_kb_workspace (workspace_id);

-- === 5. AUDIT TABLE FOR PERMISSION CHANGES ===
-- Track who granted/revoked permissions (security audit trail)
CREATE TABLE IF NOT EXISTS ap_permission_audit (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    kb_id CHAR(36) NOT NULL,
    target_user_id BIGINT UNSIGNED NULL COMMENT 'User affected by permission change',
    target_workspace_id CHAR(36) NULL COMMENT 'Workspace affected by permission change',
    action ENUM('grant', 'revoke', 'modify') NOT NULL,
    permission ENUM('read', 'write', 'admin') NULL,
    performed_by BIGINT UNSIGNED NOT NULL COMMENT 'User who performed the action',
    performed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    metadata JSON NULL COMMENT 'Additional context (old_permission, reason, etc.)',

    INDEX idx_audit_kb (kb_id),
    INDEX idx_audit_user (target_user_id),
    INDEX idx_audit_workspace (target_workspace_id),
    INDEX idx_audit_performer (performed_by),
    INDEX idx_audit_timestamp (performed_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Audit log for all permission changes';

-- === 6. DEFAULT DATA: Create "Personal" workspace for existing users ===
-- Migration safety: give existing KBs a default workspace so they remain accessible
INSERT INTO ap_workspaces (id, name, owner_user_id, created_at)
SELECT
    UUID() as id,
    CONCAT('Personal - ', u.email) as name,
    u.id as owner_user_id,
    NOW() as created_at
FROM ap_users u
WHERE NOT EXISTS (
    SELECT 1 FROM ap_workspaces w WHERE w.owner_user_id = u.id
);

-- Auto-add each user as admin of their personal workspace
INSERT INTO ap_workspace_members (workspace_id, user_id, role, joined_at)
SELECT
    w.id as workspace_id,
    w.owner_user_id as user_id,
    'admin' as role,
    NOW() as joined_at
FROM ap_workspaces w
WHERE w.name LIKE 'Personal -%'
ON DUPLICATE KEY UPDATE role = 'admin';

-- Link existing KBs to their owner's personal workspace
UPDATE ap_knowledge_bases kb
INNER JOIN ap_workspaces w ON w.owner_user_id = kb.owner_user_id
SET kb.workspace_id = w.id
WHERE kb.workspace_id IS NULL
  AND w.name LIKE 'Personal -%';

-- === 7. PERFORMANCE INDEXES ===
-- Optimize common query patterns for access control checks
CREATE INDEX idx_kb_workspace_user ON ap_knowledge_bases(workspace_id, user_id);
CREATE INDEX idx_permission_kb_lookup ON ap_kb_permissions(kb_id, permission) USING BTREE;

-- === VERIFICATION QUERIES (run manually after migration) ===
-- SELECT COUNT(*) FROM ap_workspaces; -- Should have at least 1 per existing user
-- SELECT COUNT(*) FROM ap_workspace_members; -- Should match user count (personal workspaces)
-- SELECT COUNT(*) FROM ap_knowledge_bases WHERE workspace_id IS NULL; -- Should be 0 after migration
-- SELECT kb.id, kb.name, w.name as workspace_name FROM ap_knowledge_bases kb LEFT JOIN ap_workspaces w ON kb.workspace_id = w.id;

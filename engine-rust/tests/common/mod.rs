/// Common test utilities for integration tests
///
/// Provides database setup, fixtures, and helper functions.

use sqlx::{MySql, Pool};
use std::sync::Arc;

/// Test database setup
pub async fn setup_test_db() -> Pool<MySql> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root@localhost/archivio_parlante_test".to_string());

    let pool = sqlx::MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("../db/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Clean test database
pub async fn cleanup_test_db(pool: &Pool<MySql>) {
    let tables = vec![
        "ap_graph_edges",
        "ap_graph_nodes",
        "ap_annotations",
        "ap_chat_messages",
        "ap_kb_permissions",
        "ap_workspace_members",
        "ap_documents",
        "ap_knowledge_bases",
        "ap_workspaces",
        "ap_users",
    ];

    for table in tables {
        sqlx::query(&format!("DELETE FROM {}", table))
            .execute(pool)
            .await
            .ok();
    }
}

/// Create test user
pub async fn create_test_user(pool: &Pool<MySql>, id: i64, name: &str, email: &str) -> i64 {
    sqlx::query(
        r#"
        INSERT INTO ap_users (user_id, name, email, password_hash)
        VALUES (?, ?, ?, 'test-hash')
        "#
    )
    .bind(id)
    .bind(name)
    .bind(email)
    .execute(pool)
    .await
    .expect("Failed to create test user");

    id
}

/// Create test workspace
pub async fn create_test_workspace(pool: &Pool<MySql>, id: &str, name: &str, owner_id: i64) -> String {
    sqlx::query(
        r#"
        INSERT INTO ap_workspaces (workspace_id, name, owner_user_id)
        VALUES (?, ?, ?)
        "#
    )
    .bind(id)
    .bind(name)
    .bind(owner_id)
    .execute(pool)
    .await
    .expect("Failed to create test workspace");

    id.to_string()
}

/// Create test KB
pub async fn create_test_kb(
    pool: &Pool<MySql>,
    kb_id: &str,
    name: &str,
    owner_id: i64,
    workspace_id: Option<&str>,
) -> String {
    sqlx::query(
        r#"
        INSERT INTO ap_knowledge_bases (kb_id, name, owner_user_id, workspace_id, status)
        VALUES (?, ?, ?, ?, 'active')
        "#
    )
    .bind(kb_id)
    .bind(name)
    .bind(owner_id)
    .bind(workspace_id)
    .execute(pool)
    .await
    .expect("Failed to create test KB");

    kb_id.to_string()
}

/// Add workspace member
pub async fn add_workspace_member(pool: &Pool<MySql>, workspace_id: &str, user_id: i64, role: &str) {
    sqlx::query(
        r#"
        INSERT INTO ap_workspace_members (workspace_id, user_id, role)
        VALUES (?, ?, ?)
        "#
    )
    .bind(workspace_id)
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await
    .expect("Failed to add workspace member");
}

/// Add KB permission
pub async fn add_kb_permission(pool: &Pool<MySql>, kb_id: &str, user_id: i64, permission: &str) {
    sqlx::query(
        r#"
        INSERT INTO ap_kb_permissions (kb_id, user_id, permission_type)
        VALUES (?, ?, ?)
        "#
    )
    .bind(kb_id)
    .bind(user_id)
    .bind(permission)
    .execute(pool)
    .await
    .expect("Failed to add KB permission");
}

/// Check if user has access to KB
pub async fn check_kb_access(pool: &Pool<MySql>, user_id: i64, kb_id: &str) -> bool {
    // Check direct permissions
    let direct = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ap_kb_permissions WHERE kb_id = ? AND user_id = ?"
    )
    .bind(kb_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if direct > 0 {
        return true;
    }

    // Check workspace permissions
    let workspace = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM ap_knowledge_bases kb
        INNER JOIN ap_workspace_members wm ON wm.workspace_id = kb.workspace_id
        WHERE kb.kb_id = ? AND wm.user_id = ?
        "#
    )
    .bind(kb_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if workspace > 0 {
        return true;
    }

    // Check ownership
    let owner = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ap_knowledge_bases WHERE kb_id = ? AND owner_user_id = ?"
    )
    .bind(kb_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    owner > 0
}

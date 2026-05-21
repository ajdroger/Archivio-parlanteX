/// Common test utilities for integration tests
///
/// Provides database setup, fixtures, and helper functions.

use reqwest::header::{HeaderMap, HeaderValue};
use sqlx::{MySql, Pool};
use std::env;
use std::sync::Arc;

/// Get Rust Engine internal token from environment
pub fn get_rust_token() -> String {
    env::var("RUST_ENGINE_INTERNAL_TOKEN").unwrap_or_default()
}

/// Create authenticated HTTP client with X-Internal-Token header
///
/// If RUST_ENGINE_INTERNAL_TOKEN is empty (dev mode), returns unauthenticated client.
/// Otherwise adds x-internal-token header for production auth.
pub fn authenticated_client() -> reqwest::Client {
    let token = get_rust_token();

    if token.is_empty() {
        // Dev mode: no auth required
        return reqwest::Client::new();
    }

    // Production mode: add auth header
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-internal-token",
        HeaderValue::from_str(&token).expect("Invalid RUST_ENGINE_INTERNAL_TOKEN format"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build authenticated HTTP client")
}

/// Test database setup
pub async fn setup_test_db() -> Pool<MySql> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:devpass123@localhost:3307/archivio_parlante_test".to_string());

    let pool = sqlx::MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations manually by reading SQL files
    let migrations_path = std::path::Path::new("../db/migrations");
    let mut migration_files: Vec<_> = std::fs::read_dir(migrations_path)
        .expect("Failed to read migrations directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sql"))
        .collect();

    // Sort by filename to ensure correct order
    migration_files.sort_by_key(|e| e.file_name());

    for entry in migration_files {
        let sql = std::fs::read_to_string(entry.path())
            .expect(&format!("Failed to read migration file: {:?}", entry.path()));

        // Split by semicolon and execute each statement
        for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement)
                .execute(&pool)
                .await
                .ok(); // Ignore errors (table might already exist)
        }
    }

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
        INSERT INTO ap_users (id, full_name, email, password_hash)
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
    // Ensure owner exists (create if not)
    let owner_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ap_users WHERE id = ?"
    )
    .bind(owner_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if owner_exists == 0 {
        // User doesn't exist, try to create it
        sqlx::query(
            "INSERT INTO ap_users (id, full_name, email, password_hash) VALUES (?, 'Workspace Owner', ?, 'hash') ON DUPLICATE KEY UPDATE id=id"
        )
        .bind(owner_id)
        .bind(format!("ws_owner{}@test.com", owner_id))
        .execute(pool)
        .await
        .expect(&format!("Failed to create owner user {} for workspace", owner_id));
    }

    sqlx::query(
        r#"
        INSERT INTO ap_workspaces (id, name, owner_user_id)
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
    // Ensure owner exists (create if not)
    let owner_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ap_users WHERE id = ?"
    )
    .bind(owner_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if owner_exists == 0 {
        // User doesn't exist, try to create it
        sqlx::query(
            "INSERT INTO ap_users (id, full_name, email, password_hash) VALUES (?, 'Test Owner', ?, 'hash') ON DUPLICATE KEY UPDATE id=id"
        )
        .bind(owner_id)
        .bind(format!("owner{}@test.com", owner_id))
        .execute(pool)
        .await
        .expect(&format!("Failed to create owner user {} for KB", owner_id));
    }

    sqlx::query(
        r#"
        INSERT INTO ap_knowledge_bases (id, name, owner_user_id, workspace_id)
        VALUES (?, ?, ?, ?)
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
    // Ensure user exists (in case of parallel test cleanup race conditions)
    let user_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ap_users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if user_exists == 0 {
        // User doesn't exist, recreate it
        sqlx::query(
            "INSERT INTO ap_users (id, full_name, email, password_hash) VALUES (?, 'Test User', ?, 'hash') ON DUPLICATE KEY UPDATE id=id"
        )
        .bind(user_id)
        .bind(format!("user{}@test.com", user_id))
        .execute(pool)
        .await
        .expect(&format!("Failed to recreate user {} for workspace member", user_id));
    }

    sqlx::query(
        r#"
        INSERT INTO ap_workspace_members (workspace_id, user_id, role)
        VALUES (?, ?, ?)
        "#
    )
    .bind(workspace_id)
    .bind(user_id)
    .bind(role.to_lowercase())
    .execute(pool)
    .await
    .expect("Failed to add workspace member");
}

/// Add KB permission
pub async fn add_kb_permission(pool: &Pool<MySql>, kb_id: &str, user_id: i64, permission: &str) {
    // Ensure user exists (in case of parallel test cleanup race conditions)
    let user_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ap_users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if user_exists == 0 {
        // User doesn't exist, recreate it
        sqlx::query(
            "INSERT INTO ap_users (id, full_name, email, password_hash) VALUES (?, 'Test User', ?, 'hash') ON DUPLICATE KEY UPDATE id=id"
        )
        .bind(user_id)
        .bind(format!("user{}@test.com", user_id))
        .execute(pool)
        .await
        .expect(&format!("Failed to recreate user {} for KB permission", user_id));
    }

    sqlx::query(
        r#"
        INSERT INTO ap_kb_permissions (kb_id, user_id, permission)
        VALUES (?, ?, ?)
        "#
    )
    .bind(kb_id)
    .bind(user_id)
    .bind(permission.to_lowercase())
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
        WHERE kb.id = ? AND wm.user_id = ?
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
        "SELECT COUNT(*) FROM ap_knowledge_bases WHERE id = ? AND owner_user_id = ?"
    )
    .bind(kb_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    owner > 0
}

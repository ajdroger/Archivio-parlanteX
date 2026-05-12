// ============================================================================
// Archivio Parlante — KB Access Control Middleware
// ============================================================================
// Fase 6.3.2: Multi-tenant permission enforcement
// Validates user access to knowledge bases based on workspace membership + explicit permissions

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::Row;  // Import Row trait for .get() method
use std::sync::Arc;
use tokio::time::{Duration, Instant};

use crate::{
    config::Config,
    errors::AppError,
};

/// Permission levels for KB access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// Read-only: can query KB, cannot ingest/modify
    Read,
    /// Read + Write: can query + ingest documents
    Write,
    /// Full control: can query, ingest, delete, manage permissions
    Admin,
}

impl Permission {
    /// Check if this permission level includes another permission
    /// Admin > Write > Read
    pub fn includes(&self, other: &Permission) -> bool {
        match (self, other) {
            (Permission::Admin, _) => true,
            (Permission::Write, Permission::Write | Permission::Read) => true,
            (Permission::Read, Permission::Read) => true,
            _ => false,
        }
    }
}

/// KB Access Control Middleware State
#[derive(Clone)]
pub struct KbAccessMiddleware {
    config: Arc<Config>,
    redis_client: Option<redis::Client>,
    db_pool: sqlx::MySqlPool,
}

impl KbAccessMiddleware {
    /// Create new KB access control middleware
    pub fn new(config: Arc<Config>, db_pool: sqlx::MySqlPool) -> Self {
        // Initialize Redis client for permission caching
        let redis_client = redis::Client::open(config.redis_url.as_str())
            .ok();

        Self {
            config,
            redis_client,
            db_pool,
        }
    }

    /// Check if user has required permission on KB
    /// Returns Ok(()) if allowed, Err(AppError) if denied
    pub async fn check_permission(
        &self,
        user_id: u64,
        kb_id: &str,
        required: Permission,
    ) -> Result<(), AppError> {
        // Try cache first (Redis, TTL 5 minutes)
        if let Some(cached) = self.check_cache(user_id, kb_id).await {
            return if cached.includes(&required) {
                Ok(())
            } else {
                Err(AppError::Forbidden(format!(
                    "Insufficient permission: required {:?}, have {:?}",
                    required, cached
                )))
            };
        }

        // Cache miss: query MySQL for permission
        let permission = self.query_permission(user_id, kb_id).await?;

        // Update cache
        self.set_cache(user_id, kb_id, permission).await;

        // Validate permission level
        if permission.includes(&required) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Insufficient permission: required {:?}, have {:?}",
                required, permission
            )))
        }
    }

    /// Query MySQL for user's permission on KB
    /// Checks (in order of precedence):
    /// 1. Direct user-level permission in ap_kb_permissions
    /// 2. Workspace-level permission (if user is member of KB's workspace)
    /// 3. KB owner (always has Admin permission)
    /// 4. Workspace admin (has Admin on all workspace KBs)
    async fn query_permission(&self, user_id: u64, kb_id: &str) -> Result<Permission, AppError> {
        // Query with 4-tier permission check using COALESCE
        let result = sqlx::query(
            r#"
            SELECT
                COALESCE(
                    -- 1. Direct user permission
                    (SELECT permission FROM ap_kb_permissions
                     WHERE kb_id = ? AND user_id = ? LIMIT 1),

                    -- 2. Workspace permission (user is member of workspace with KB shared)
                    (SELECT kbp.permission FROM ap_kb_permissions kbp
                     INNER JOIN ap_workspace_members wm ON kbp.workspace_id = wm.workspace_id
                     WHERE kbp.kb_id = ? AND wm.user_id = ? LIMIT 1),

                    -- 3. KB owner (implicit admin)
                    (SELECT 'admin' FROM ap_knowledge_bases
                     WHERE id = ? AND owner_user_id = ? LIMIT 1),

                    -- 4. Workspace admin (implicit admin on all workspace KBs)
                    (SELECT 'admin' FROM ap_workspace_members wm
                     INNER JOIN ap_knowledge_bases kb ON wm.workspace_id = kb.workspace_id
                     WHERE kb.id = ? AND wm.user_id = ? AND wm.role = 'admin' LIMIT 1)
                ) as permission
            "#
        )
        .bind(kb_id).bind(user_id as i64)  // Tier 1: direct permission
        .bind(kb_id).bind(user_id as i64)  // Tier 2: workspace permission
        .bind(kb_id).bind(user_id as i64)  // Tier 3: KB owner
        .bind(kb_id).bind(user_id as i64)  // Tier 4: workspace admin
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error querying KB permission: {}", e);
            AppError::InternalError(format!("Database error: {}", e))
        })?;

        // Parse permission string to enum
        let permission_str: Option<String> = result.and_then(|row| row.get::<Option<String>, _>("permission"));
        match permission_str {
            Some(perm) => match perm.as_str() {
                "read" => Ok(Permission::Read),
                "write" => Ok(Permission::Write),
                "admin" => Ok(Permission::Admin),
                _ => {
                    tracing::error!("Invalid permission value in database: {}", perm);
                    Err(AppError::Forbidden("No valid permission found".to_string()))
                }
            },
            None => {
                tracing::debug!(
                    "No permission found for user_id={} on kb_id={}",
                    user_id,
                    kb_id
                );
                Err(AppError::Forbidden(
                    "You do not have permission to access this knowledge base".to_string()
                ))
            }
        }
    }

    /// Check Redis cache for permission
    /// Returns Some(Permission) if cached, None if cache miss
    async fn check_cache(&self, user_id: u64, kb_id: &str) -> Option<Permission> {
        let client = self.redis_client.as_ref()?;
        let mut conn = client.get_async_connection().await.ok()?;

        let cache_key = format!("kb_perm:{}:{}", user_id, kb_id);

        let cached: Option<String> = redis::cmd("GET")
            .arg(&cache_key)
            .query_async(&mut conn)
            .await
            .ok()?;

        cached.and_then(|s| match s.as_str() {
            "read" => Some(Permission::Read),
            "write" => Some(Permission::Write),
            "admin" => Some(Permission::Admin),
            _ => None,
        })
    }

    /// Set Redis cache for permission (TTL 5 minutes)
    async fn set_cache(&self, user_id: u64, kb_id: &str, permission: Permission) {
        if let Some(client) = &self.redis_client {
            if let Ok(mut conn) = client.get_async_connection().await {
                let cache_key = format!("kb_perm:{}:{}", user_id, kb_id);
                let cache_value = match permission {
                    Permission::Read => "read",
                    Permission::Write => "write",
                    Permission::Admin => "admin",
                };

                let _: Result<(), redis::RedisError> = redis::cmd("SETEX")
                    .arg(&cache_key)
                    .arg(300) // TTL 5 minutes
                    .arg(cache_value)
                    .query_async(&mut conn)
                    .await;
            }
        }
    }
}

/// Axum middleware function for KB access control
/// Extracts kb_id from request path and validates user permission
pub async fn kb_access_middleware(
    State(middleware): State<Arc<KbAccessMiddleware>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let start = Instant::now();

    // Extract kb_id from path parameters
    let kb_id_opt = extract_kb_id_from_path(request.uri().path());

    // If no kb_id in path (e.g., /query, /ingest), skip this middleware
    // and let the handler check permissions (or rely on internal auth)
    let kb_id = match kb_id_opt {
        Some(id) => id,
        None => return Ok(next.run(request).await),
    };

    // Extract user_id from request extensions (set by auth middleware)
    let user_id = request
        .extensions()
        .get::<u64>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("User ID not found in request".to_string()))?;

    // Determine required permission based on HTTP method
    let required_permission = match *request.method() {
        axum::http::Method::GET => Permission::Read,
        axum::http::Method::POST | axum::http::Method::PUT => Permission::Write,
        axum::http::Method::DELETE => Permission::Admin,
        _ => Permission::Read,
    };

    // Check permission
    middleware.check_permission(user_id, &kb_id, required_permission).await?;

    // Store kb_id in request extensions for downstream handlers
    request.extensions_mut().insert(kb_id.clone());

    // Proceed to next middleware/handler
    let response = next.run(request).await;

    // Log access check timing
    let elapsed = start.elapsed();
    tracing::debug!(
        "KB access check: user_id={}, kb_id={}, permission={:?}, elapsed={:?}",
        user_id,
        kb_id,
        required_permission,
        elapsed
    );

    Ok(response)
}

/// Extract kb_id from request path
/// Supports patterns: /kb/:kb_id/*, /query?kb_id=:kb_id, etc.
fn extract_kb_id_from_path(path: &str) -> Option<String> {
    // Pattern 1: /kb/:kb_id/documents
    if let Some(caps) = regex::Regex::new(r"/kb/([a-f0-9\-]+)")
        .ok()?
        .captures(path)
    {
        return Some(caps.get(1)?.as_str().to_string());
    }

    // Pattern 2: /query, /ingest, /compare (kb_id in request body, will be checked there)
    // For these routes, we skip middleware and check permission in handler
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_includes() {
        assert!(Permission::Admin.includes(&Permission::Read));
        assert!(Permission::Admin.includes(&Permission::Write));
        assert!(Permission::Admin.includes(&Permission::Admin));

        assert!(Permission::Write.includes(&Permission::Read));
        assert!(Permission::Write.includes(&Permission::Write));
        assert!(!Permission::Write.includes(&Permission::Admin));

        assert!(Permission::Read.includes(&Permission::Read));
        assert!(!Permission::Read.includes(&Permission::Write));
        assert!(!Permission::Read.includes(&Permission::Admin));
    }

    #[test]
    fn test_extract_kb_id() {
        assert_eq!(
            extract_kb_id_from_path("/kb/abc-123-def/documents"),
            Some("abc-123-def".to_string())
        );

        assert_eq!(
            extract_kb_id_from_path("/kb/550e8400-e29b-41d4-a716-446655440000/stats"),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );

        assert_eq!(extract_kb_id_from_path("/query"), None);
        assert_eq!(extract_kb_id_from_path("/health"), None);
    }
}

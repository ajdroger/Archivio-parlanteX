/// Presence tracking for collaborative editing
///
/// Tracks which users are currently viewing/editing a document using Redis
/// sorted sets with timestamp-based cleanup.

use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::{AppError, Result};

/// Presence tracker using Redis sorted sets
pub struct PresenceTracker {
    redis_client: RedisClient,
    stale_timeout_secs: u64,
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub avatar_url: Option<String>,
}

impl PresenceTracker {
    /// Create new presence tracker
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL
    /// * `stale_timeout_secs` - Timeout in seconds before a user is considered offline (default: 60)
    pub fn new(redis_url: String, stale_timeout_secs: Option<u64>) -> Result<Self> {
        let redis_client =
            RedisClient::open(redis_url).map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(Self {
            redis_client,
            stale_timeout_secs: stale_timeout_secs.unwrap_or(60),
        })
    }

    /// User joins document (starts viewing/editing)
    ///
    /// Returns list of all current users viewing the document
    ///
    /// # Arguments
    /// * `kb_id` - Knowledge base ID
    /// * `doc_id` - Document ID
    /// * `user` - User joining
    pub async fn join(&self, kb_id: &str, doc_id: &str, user: &User) -> Result<Vec<User>> {
        let key = Self::presence_key(kb_id, doc_id);
        let now = Self::current_timestamp();

        // Add user with current timestamp as score
        let user_json = serde_json::to_string(user)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize user: {}", e)))?;

        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::InternalError(format!("Redis connection failed: {}", e)))?;

        conn.zadd::<_, _, _, ()>(&key, &user_json, now as f64)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis ZADD failed: {}", e)))?;

        tracing::info!(
            user_id = user.id,
            kb_id = %kb_id,
            doc_id = %doc_id,
            "User joined document"
        );

        // Return all active users
        self.get_active_users(kb_id, doc_id).await
    }

    /// User leaves document
    ///
    /// # Arguments
    /// * `kb_id` - Knowledge base ID
    /// * `doc_id` - Document ID
    /// * `user_id` - User ID leaving
    pub async fn leave(&self, kb_id: &str, doc_id: &str, user_id: u64) -> Result<()> {
        let key = Self::presence_key(kb_id, doc_id);

        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::InternalError(format!("Redis connection failed: {}", e)))?;

        // Get all members to find the one to remove
        let members: Vec<String> = conn
            .zrange(&key, 0, -1)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis ZRANGE failed: {}", e)))?;

        // Find and remove user
        for member in members {
            if let Ok(user) = serde_json::from_str::<User>(&member) {
                if user.id == user_id {
                    conn.zrem::<_, _, ()>(&key, &member)
                        .await
                        .map_err(|e| AppError::InternalError(format!("Redis ZREM failed: {}", e)))?;

                    tracing::info!(
                        user_id = user_id,
                        kb_id = %kb_id,
                        doc_id = %doc_id,
                        "User left document"
                    );

                    break;
                }
            }
        }

        Ok(())
    }

    /// Update user heartbeat (keep-alive)
    ///
    /// # Arguments
    /// * `kb_id` - Knowledge base ID
    /// * `doc_id` - Document ID
    /// * `user_id` - User ID
    pub async fn heartbeat(&self, kb_id: &str, doc_id: &str, user_id: u64) -> Result<()> {
        let key = Self::presence_key(kb_id, doc_id);
        let now = Self::current_timestamp();

        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::InternalError(format!("Redis connection failed: {}", e)))?;

        // Get all members to find the user to update
        let members: Vec<String> = conn
            .zrange(&key, 0, -1)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis ZRANGE failed: {}", e)))?;

        // Find and update user timestamp
        for member in members {
            if let Ok(user) = serde_json::from_str::<User>(&member) {
                if user.id == user_id {
                    // Update score (timestamp)
                    conn.zadd::<_, _, _, ()>(&key, &member, now as f64)
                        .await
                        .map_err(|e| AppError::InternalError(format!("Redis ZADD failed: {}", e)))?;

                    tracing::debug!(
                        user_id = user_id,
                        kb_id = %kb_id,
                        doc_id = %doc_id,
                        "Heartbeat updated"
                    );

                    break;
                }
            }
        }

        Ok(())
    }

    /// Get list of active users viewing the document
    ///
    /// # Arguments
    /// * `kb_id` - Knowledge base ID
    /// * `doc_id` - Document ID
    pub async fn get_active_users(&self, kb_id: &str, doc_id: &str) -> Result<Vec<User>> {
        let key = Self::presence_key(kb_id, doc_id);

        // Clean up stale users first
        self.cleanup_stale_users(kb_id, doc_id).await?;

        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::InternalError(format!("Redis connection failed: {}", e)))?;

        let members: Vec<String> = conn
            .zrange(&key, 0, -1)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis ZRANGE failed: {}", e)))?;

        let users: Vec<User> = members
            .iter()
            .filter_map(|m| serde_json::from_str(m).ok())
            .collect();

        Ok(users)
    }

    /// Clean up stale users (last seen > timeout)
    async fn cleanup_stale_users(&self, kb_id: &str, doc_id: &str) -> Result<()> {
        let key = Self::presence_key(kb_id, doc_id);
        let now = Self::current_timestamp();
        let cutoff = now - self.stale_timeout_secs;

        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::InternalError(format!("Redis connection failed: {}", e)))?;

        // Remove users with timestamp < cutoff
        let removed: usize = conn
            .zrembyscore(&key, 0.0, cutoff as f64)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis ZREMBYSCORE failed: {}", e)))?;

        if removed > 0 {
            tracing::info!(
                removed = removed,
                kb_id = %kb_id,
                doc_id = %doc_id,
                "Cleaned up stale users"
            );
        }

        Ok(())
    }

    /// Generate presence key for kb_id:doc_id
    fn presence_key(kb_id: &str, doc_id: &str) -> String {
        format!("presence:{}:{}", kb_id, doc_id)
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_key() {
        let key = PresenceTracker::presence_key("kb_123", "doc_456");
        assert_eq!(key, "presence:kb_123:doc_456");
    }

    #[test]
    fn test_current_timestamp() {
        let ts = PresenceTracker::current_timestamp();
        assert!(ts > 1700000000); // After 2023
        assert!(ts < 2000000000); // Before 2033
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        };

        let json = serde_json::to_string(&user).expect("Serialization failed");
        let deserialized: User = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.name, deserialized.name);
        assert_eq!(user.avatar_url, deserialized.avatar_url);
    }
}

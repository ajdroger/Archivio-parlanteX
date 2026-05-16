/// Annotation broadcaster using Redis pub/sub
///
/// Broadcasts annotation updates to all connected WebSocket clients
/// viewing the same document.

use redis::{aio::MultiplexedConnection, AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::errors::{AppError, Result};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Annotation created
    #[serde(rename = "annotation.created")]
    AnnotationCreated { annotation: Annotation },

    /// Annotation updated
    #[serde(rename = "annotation.updated")]
    AnnotationUpdated { annotation: Annotation },

    /// Annotation deleted
    #[serde(rename = "annotation.deleted")]
    AnnotationDeleted { annotation_id: String },

    /// Presence update (user joined/left)
    #[serde(rename = "presence.update")]
    PresenceUpdate { users: Vec<PresenceUser> },

    /// Heartbeat (keep-alive)
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: i64 },
}

/// Annotation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub chunk_id: String,
    pub user: AnnotationUser,
    pub text: String,
    pub position: Position,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// User info in annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationUser {
    pub id: u64,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// Text position in chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub start: u32,
    pub end: u32,
}

/// Presence user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUser {
    pub id: u64,
    pub name: String,
    pub avatar_url: Option<String>,
    pub last_seen: String,
}

/// Annotation broadcaster with Redis pub/sub
pub struct AnnotationBroadcaster {
    redis_client: RedisClient,
    connection: Arc<RwLock<MultiplexedConnection>>,
}

impl AnnotationBroadcaster {
    /// Create new broadcaster
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL
    pub async fn new(redis_url: String) -> Result<Self> {
        let redis_client =
            RedisClient::open(redis_url).map_err(|e| AppError::InternalError(e.to_string()))?;

        let connection = redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::InternalError(format!("Redis connection failed: {}", e)))?;

        Ok(Self {
            redis_client,
            connection: Arc::new(RwLock::new(connection)),
        })
    }

    /// Broadcast message to all clients subscribed to kb_id:doc_id channel
    ///
    /// # Arguments
    /// * `kb_id` - Knowledge base ID
    /// * `doc_id` - Document ID
    /// * `message` - Message to broadcast
    pub async fn broadcast(&self, kb_id: &str, doc_id: &str, message: WsMessage) -> Result<()> {
        let channel = Self::channel_name(kb_id, doc_id);

        let message_json = serde_json::to_string(&message)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize message: {}", e)))?;

        let mut conn = self.connection.write().await;

        conn.publish::<_, _, ()>(&channel, message_json)
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Redis PUBLISH failed: {}", e))
            })?;

        tracing::debug!(
            channel = %channel,
            message_type = ?message,
            "Broadcast message sent"
        );

        Ok(())
    }

    /// Subscribe to kb_id:doc_id channel
    ///
    /// Returns a pubsub connection that can be used to receive messages
    pub async fn subscribe(&self, kb_id: &str, doc_id: &str) -> Result<redis::aio::PubSub> {
        let channel = Self::channel_name(kb_id, doc_id);

        let mut pubsub = self
            .redis_client
            .get_async_pubsub()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create pubsub: {}", e)))?;

        pubsub
            .subscribe(&channel)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis SUBSCRIBE failed: {}", e)))?;

        tracing::info!(
            channel = %channel,
            "Subscribed to collaboration channel"
        );

        Ok(pubsub)
    }

    /// Generate channel name for kb_id:doc_id
    fn channel_name(kb_id: &str, doc_id: &str) -> String {
        format!("ws:collab:{}:{}", kb_id, doc_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_name() {
        let name = AnnotationBroadcaster::channel_name("kb_123", "doc_456");
        assert_eq!(name, "ws:collab:kb_123:doc_456");
    }

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::AnnotationCreated {
            annotation: Annotation {
                id: "ann_001".to_string(),
                chunk_id: "chunk_001".to_string(),
                user: AnnotationUser {
                    id: 1,
                    name: "Alice".to_string(),
                    avatar_url: None,
                },
                text: "Test annotation".to_string(),
                position: Position { start: 0, end: 10 },
                created_at: "2026-05-08T10:00:00Z".to_string(),
                updated_at: None,
            },
        };

        let json = serde_json::to_string(&msg).expect("Serialization failed");
        assert!(json.contains("annotation.created"));
        assert!(json.contains("Test annotation"));
    }

    #[test]
    fn test_presence_update_serialization() {
        let msg = WsMessage::PresenceUpdate {
            users: vec![PresenceUser {
                id: 1,
                name: "Alice".to_string(),
                avatar_url: None,
                last_seen: "2026-05-08T10:00:00Z".to_string(),
            }],
        };

        let json = serde_json::to_string(&msg).expect("Serialization failed");
        assert!(json.contains("presence.update"));
        assert!(json.contains("Alice"));
    }
}

/// WebSocket handler for collaborative annotations
///
/// Handles WebSocket connections for real-time collaboration with:
/// - Bidirectional message flow (client ↔ server)
/// - Redis pub/sub broadcasting
/// - Presence tracking (join/leave/heartbeat)
/// - Annotation CRUD operations

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::errors::{AppError, Result};
use crate::routes::ingest::AppState;
use crate::websocket::broadcaster::{
    Annotation, AnnotationBroadcaster, AnnotationUser, Position, WsMessage,
};
use crate::websocket::presence::{PresenceTracker, User};

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// Knowledge base ID
    kb_id: String,

    /// Document ID
    doc_id: String,

    /// User ID (from JWT)
    user_id: u64,

    /// User name
    user_name: String,

    /// Optional avatar URL
    avatar_url: Option<String>,
}

/// Client → Server messages
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    /// Create new annotation
    #[serde(rename = "annotation.create")]
    CreateAnnotation {
        chunk_id: String,
        text: String,
        position: Position,
    },

    /// Update existing annotation
    #[serde(rename = "annotation.update")]
    UpdateAnnotation {
        annotation_id: String,
        text: String,
    },

    /// Delete annotation
    #[serde(rename = "annotation.delete")]
    DeleteAnnotation { annotation_id: String },

    /// Heartbeat (keep-alive)
    #[serde(rename = "heartbeat")]
    Heartbeat,
}

/// WebSocket upgrade handler
///
/// Upgrades HTTP connection to WebSocket and spawns handler tasks
pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    tracing::info!(
        user_id = query.user_id,
        kb_id = %query.kb_id,
        doc_id = %query.doc_id,
        "WebSocket connection requested"
    );

    // KB access control check - verify user has access to KB
    if let Err(e) = check_kb_access(&state, query.user_id, &query.kb_id).await {
        tracing::warn!(
            user_id = query.user_id,
            kb_id = %query.kb_id,
            error = %e,
            "KB access denied for WebSocket connection"
        );

        return (
            StatusCode::FORBIDDEN,
            format!("Access denied to KB: {}", e),
        )
            .into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, query, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, query: WsQuery, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Create broadcaster and presence tracker
    let broadcaster = match AnnotationBroadcaster::new(state.config.redis_url.clone()).await {
        Ok(b) => Arc::new(b),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create broadcaster");
            let _ = sender
                .send(Message::Text(
                    serde_json::json!({
                        "type": "error",
                        "message": "Failed to initialize connection"
                    })
                    .to_string(),
                ))
                .await;
            return;
        }
    };

    let presence = match PresenceTracker::new(state.config.redis_url.clone(), Some(60)) {
        Ok(p) => Arc::new(p),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create presence tracker");
            return;
        }
    };

    // User info
    let user = User {
        id: query.user_id,
        name: query.user_name.clone(),
        avatar_url: query.avatar_url.clone(),
    };

    // Join document (add to presence)
    let active_users = match presence.join(&query.kb_id, &query.doc_id, &user).await {
        Ok(users) => users,
        Err(e) => {
            tracing::error!(error = %e, "Failed to join presence");
            return;
        }
    };

    // Broadcast presence update
    let presence_users = active_users
        .iter()
        .map(|u| crate::websocket::broadcaster::PresenceUser {
            id: u.id,
            name: u.name.clone(),
            avatar_url: u.avatar_url.clone(),
            last_seen: chrono::Utc::now().to_rfc3339(),
        })
        .collect();

    if let Err(e) = broadcaster
        .broadcast(
            &query.kb_id,
            &query.doc_id,
            WsMessage::PresenceUpdate {
                users: presence_users,
            },
        )
        .await
    {
        tracing::error!(error = %e, "Failed to broadcast presence");
    }

    // Subscribe to Redis pub/sub
    let mut pubsub = match broadcaster.subscribe(&query.kb_id, &query.doc_id).await {
        Ok(ps) => ps,
        Err(e) => {
            tracing::error!(error = %e, "Failed to subscribe to pub/sub");
            return;
        }
    };

    // Create channel for sending messages to client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Spawn task to forward Redis pub/sub messages to client
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut pubsub_stream = pubsub.on_message();
        while let Some(msg) = pubsub_stream.next().await {
            if let Ok(payload) = msg.get_payload::<String>() {
                let _ = tx_clone.send(payload);
            }
        }
    });

    // Spawn task to send messages from channel to WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming client messages
    let user_id = query.user_id;
    let kb_id = query.kb_id.clone();
    let doc_id = query.doc_id.clone();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_client_message(
                    &text,
                    &query,
                    &state,
                    &broadcaster,
                    &presence,
                    &user,
                )
                .await
                {
                    tracing::error!(error = %e, "Failed to handle client message");
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!(user_id = user_id, "WebSocket connection closed");
                break;
            }
            Err(e) => {
                tracing::error!(error = %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }

    // Cleanup: remove user from presence
    if let Err(e) = presence.leave(&kb_id, &doc_id, user_id).await {
        tracing::error!(error = %e, "Failed to leave presence");
    }

    // Broadcast updated presence
    if let Ok(active_users) = presence.get_active_users(&kb_id, &doc_id).await {
        let presence_users = active_users
            .iter()
            .map(|u| crate::websocket::broadcaster::PresenceUser {
                id: u.id,
                name: u.name.clone(),
                avatar_url: u.avatar_url.clone(),
                last_seen: chrono::Utc::now().to_rfc3339(),
            })
            .collect();

        let _ = broadcaster
            .broadcast(&kb_id, &doc_id, WsMessage::PresenceUpdate { users: presence_users })
            .await;
    }
}

/// Handle client message
async fn handle_client_message(
    text: &str,
    query: &WsQuery,
    state: &AppState,
    broadcaster: &Arc<AnnotationBroadcaster>,
    presence: &Arc<PresenceTracker>,
    user: &User,
) -> Result<()> {
    let msg: ClientMessage = serde_json::from_str(text)
        .map_err(|e| AppError::BadRequest(format!("Invalid message format: {}", e)))?;

    match msg {
        ClientMessage::CreateAnnotation {
            chunk_id,
            text,
            position,
        } => {
            let annotation = create_annotation(query, state, &chunk_id, &text, &position, user).await?;
            broadcaster
                .broadcast(
                    &query.kb_id,
                    &query.doc_id,
                    WsMessage::AnnotationCreated { annotation },
                )
                .await?;
        }
        ClientMessage::UpdateAnnotation {
            annotation_id,
            text,
        } => {
            let annotation = update_annotation(state, &annotation_id, &text).await?;
            broadcaster
                .broadcast(
                    &query.kb_id,
                    &query.doc_id,
                    WsMessage::AnnotationUpdated { annotation },
                )
                .await?;
        }
        ClientMessage::DeleteAnnotation { annotation_id } => {
            delete_annotation(state, &annotation_id).await?;
            broadcaster
                .broadcast(
                    &query.kb_id,
                    &query.doc_id,
                    WsMessage::AnnotationDeleted { annotation_id },
                )
                .await?;
        }
        ClientMessage::Heartbeat => {
            presence
                .heartbeat(&query.kb_id, &query.doc_id, user.id)
                .await?;
        }
    }

    Ok(())
}

/// Create annotation in database
async fn create_annotation(
    query: &WsQuery,
    state: &AppState,
    chunk_id: &str,
    text: &str,
    position: &Position,
    user: &User,
) -> Result<Annotation> {
    let annotation_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO ap_annotations (
            id, kb_id, doc_id, chunk_id, user_id, text, position_start, position_end
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&annotation_id)
    .bind(&query.kb_id)
    .bind(&query.doc_id)
    .bind(chunk_id)
    .bind(user.id as i64)
    .bind(text)
    .bind(position.start as i32)
    .bind(position.end as i32)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to create annotation: {}", e)))?;

    let created_at = chrono::Utc::now().to_rfc3339();

    Ok(Annotation {
        id: annotation_id,
        chunk_id: chunk_id.to_string(),
        user: AnnotationUser {
            id: user.id,
            name: user.name.clone(),
            avatar_url: user.avatar_url.clone(),
        },
        text: text.to_string(),
        position: position.clone(),
        created_at,
        updated_at: None,
    })
}

/// Update annotation in database
async fn update_annotation(
    state: &AppState,
    annotation_id: &str,
    text: &str,
) -> Result<Annotation> {
    sqlx::query(
        r#"
        UPDATE ap_annotations
        SET text = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND deleted_at IS NULL
        "#
    )
    .bind(text)
    .bind(annotation_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to update annotation: {}", e)))?;

    // Fetch updated annotation
    let row = sqlx::query(
        r#"
        SELECT a.id, a.chunk_id, a.text, a.position_start, a.position_end,
               a.created_at, a.updated_at, a.user_id,
               u.name as user_name, u.avatar_url
        FROM ap_annotations a
        JOIN ap_users u ON a.user_id = u.id
        WHERE a.id = ?
        "#
    )
    .bind(annotation_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::NotFound(format!("Annotation not found: {}", e)))?;

    Ok(Annotation {
        id: row.get("id"),
        chunk_id: row.get("chunk_id"),
        user: AnnotationUser {
            id: row.get::<i64, _>("user_id") as u64,
            name: row.get("user_name"),
            avatar_url: row.get("avatar_url"),
        },
        text: row.get("text"),
        position: Position {
            start: row.get::<i32, _>("position_start") as u32,
            end: row.get::<i32, _>("position_end") as u32,
        },
        created_at: row.get::<chrono::NaiveDateTime, _>("created_at").and_utc().to_rfc3339(),
        updated_at: row.get::<Option<chrono::NaiveDateTime>, _>("updated_at").map(|dt| dt.and_utc().to_rfc3339()),
    })
}

/// Delete annotation (soft delete)
async fn delete_annotation(state: &AppState, annotation_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE ap_annotations
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE id = ? AND deleted_at IS NULL
        "#
    )
    .bind(annotation_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to delete annotation: {}", e)))?;

    Ok(())
}

/// Check if user has access to KB
async fn check_kb_access(state: &AppState, user_id: u64, kb_id: &str) -> Result<()> {
    // Check 1: Direct permissions (ap_kb_permissions)
    let direct_permission = sqlx::query(
        r#"
        SELECT permission_type
        FROM ap_kb_permissions
        WHERE kb_id = ? AND user_id = ?
        "#
    )
    .bind(kb_id)
    .bind(user_id as i64)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to check direct permissions: {}", e)))?;

    if direct_permission.is_some() {
        return Ok(()); // User has direct permission
    }

    // Check 2: Workspace permissions (ap_workspace_members + ap_knowledge_bases)
    let workspace_permission = sqlx::query(
        r#"
        SELECT wm.role
        FROM ap_knowledge_bases kb
        INNER JOIN ap_workspace_members wm ON wm.workspace_id = kb.workspace_id
        WHERE kb.kb_id = ? AND wm.user_id = ?
        "#
    )
    .bind(kb_id)
    .bind(user_id as i64)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to check workspace permissions: {}", e)))?;

    if workspace_permission.is_some() {
        return Ok(()); // User is workspace member
    }

    // Check 3: Ownership
    let is_owner = sqlx::query(
        r#"
        SELECT 1
        FROM ap_knowledge_bases
        WHERE kb_id = ? AND owner_user_id = ?
        "#
    )
    .bind(kb_id)
    .bind(user_id as i64)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalError(format!("Failed to check ownership: {}", e)))?;

    if is_owner.is_some() {
        return Ok(()); // User is owner
    }

    // No access found
    Err(AppError::Forbidden("You do not have access to this knowledge base".to_string()))
}

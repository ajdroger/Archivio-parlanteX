/// WebSocket collaboration module
///
/// Fase 6.4 - Real-time Collaborative Annotation
///
/// Implements WebSocket-based real-time collaboration for document annotations
/// with presence tracking and Redis pub/sub broadcasting.

pub mod broadcaster;
pub mod handler;
pub mod presence;

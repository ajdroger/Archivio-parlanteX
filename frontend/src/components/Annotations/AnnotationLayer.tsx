/**
 * AnnotationLayer Component
 * Fase 6.4 - Real-time Collaborative Annotation
 *
 * Displays and manages collaborative annotations on PDF chunks with:
 * - Real-time updates via WebSocket
 * - Presence indicators showing active users
 * - Annotation highlights with popovers
 * - Modal for creating/editing annotations
 */

import React, { useState, useEffect, useCallback } from 'react';
import { useCollaboration, Annotation, PresenceUser, WsMessage } from '@/lib/websocket';

export interface AnnotationLayerProps {
  /** Knowledge base ID */
  kbId: string;

  /** Document ID */
  docId: string;

  /** Chunk ID */
  chunkId: string;

  /** Chunk text */
  chunkText: string;

  /** Engine URL for WebSocket */
  engineUrl: string;

  /** JWT token */
  token: string;

  /** Current user ID */
  userId: number;

  /** Current user name */
  userName: string;

  /** Current user avatar URL */
  avatarUrl?: string;
}

/**
 * AnnotationLayer - Overlay for collaborative annotations
 */
export const AnnotationLayer: React.FC<AnnotationLayerProps> = ({
  kbId,
  docId,
  chunkId,
  chunkText,
  engineUrl,
  token,
  userId,
  userName,
  avatarUrl,
}) => {
  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [selectedText, setSelectedText] = useState<{
    text: string;
    start: number;
    end: number;
  } | null>(null);
  const [showModal, setShowModal] = useState(false);
  const [annotationText, setAnnotationText] = useState('');

  // WebSocket collaboration
  const { client, isConnected, presenceUsers } = useCollaboration({
    engineUrl,
    kbId,
    docId,
    token,
    userId,
    userName,
    avatarUrl,
  });

  // Handle incoming WebSocket messages
  useEffect(() => {
    const unsubscribe = client.onMessage((message: WsMessage) => {
      if (message.type === 'annotation.created' && message.annotation) {
        if (message.annotation.chunk_id === chunkId) {
          setAnnotations((prev) => [...prev, message.annotation!]);
        }
      } else if (message.type === 'annotation.updated' && message.annotation) {
        setAnnotations((prev) =>
          prev.map((ann) =>
            ann.id === message.annotation!.id ? message.annotation! : ann
          )
        );
      } else if (message.type === 'annotation.deleted' && message.annotation_id) {
        setAnnotations((prev) =>
          prev.filter((ann) => ann.id !== message.annotation_id)
        );
      }
    });

    return unsubscribe;
  }, [client, chunkId]);

  // Handle text selection
  const handleTextSelect = useCallback(() => {
    const selection = window.getSelection();
    if (!selection || selection.isCollapsed) {
      setSelectedText(null);
      return;
    }

    const text = selection.toString().trim();
    if (text.length === 0) {
      return;
    }

    // Calculate position in chunk
    const range = selection.getRangeAt(0);
    const preSelectionRange = range.cloneRange();
    preSelectionRange.selectNodeContents(range.startContainer.parentElement!);
    preSelectionRange.setEnd(range.startContainer, range.startOffset);
    const start = preSelectionRange.toString().length;
    const end = start + text.length;

    setSelectedText({ text, start, end });
  }, []);

  // Create annotation
  const handleCreateAnnotation = useCallback(() => {
    if (!selectedText || !annotationText.trim()) {
      return;
    }

    client.createAnnotation(chunkId, annotationText.trim(), {
      start: selectedText.start,
      end: selectedText.end,
    });

    // Reset state
    setShowModal(false);
    setAnnotationText('');
    setSelectedText(null);
    window.getSelection()?.removeAllRanges();
  }, [client, chunkId, selectedText, annotationText]);

  // Delete annotation
  const handleDeleteAnnotation = useCallback(
    (annotationId: string) => {
      client.deleteAnnotation(annotationId);
    },
    [client]
  );

  // Render chunk text with annotation highlights
  const renderAnnotatedText = useCallback(() => {
    if (annotations.length === 0) {
      return <div className="text-gray-800">{chunkText}</div>;
    }

    // Sort annotations by start position
    const sortedAnnotations = [...annotations].sort(
      (a, b) => a.position.start - b.position.start
    );

    const segments: React.ReactNode[] = [];
    let lastIndex = 0;

    sortedAnnotations.forEach((annotation, index) => {
      // Add text before annotation
      if (annotation.position.start > lastIndex) {
        segments.push(
          <span key={`text-${index}`}>
            {chunkText.slice(lastIndex, annotation.position.start)}
          </span>
        );
      }

      // Add annotated text
      const annotatedText = chunkText.slice(
        annotation.position.start,
        annotation.position.end
      );

      segments.push(
        <span
          key={`annotation-${annotation.id}`}
          className="bg-yellow-200 hover:bg-yellow-300 cursor-pointer relative group"
          title={annotation.text}
        >
          {annotatedText}
          {/* Popover on hover */}
          <div className="absolute z-10 hidden group-hover:block bg-white border border-gray-300 rounded-lg shadow-lg p-3 mt-1 min-w-[200px] max-w-[300px]">
            <div className="flex items-start justify-between mb-2">
              <div className="flex items-center gap-2">
                {annotation.user.avatar_url && (
                  <img
                    src={annotation.user.avatar_url}
                    alt={annotation.user.name}
                    className="w-6 h-6 rounded-full"
                  />
                )}
                <span className="font-semibold text-sm">
                  {annotation.user.name}
                </span>
              </div>
              {annotation.user.id === userId && (
                <button
                  onClick={() => handleDeleteAnnotation(annotation.id)}
                  className="text-red-500 hover:text-red-700 text-xs"
                >
                  Delete
                </button>
              )}
            </div>
            <p className="text-sm text-gray-700">{annotation.text}</p>
            <p className="text-xs text-gray-500 mt-1">
              {new Date(annotation.created_at).toLocaleString()}
            </p>
          </div>
        </span>
      );

      lastIndex = annotation.position.end;
    });

    // Add remaining text
    if (lastIndex < chunkText.length) {
      segments.push(
        <span key="text-end">{chunkText.slice(lastIndex)}</span>
      );
    }

    return <div className="text-gray-800">{segments}</div>;
  }, [annotations, chunkText, userId, handleDeleteAnnotation]);

  return (
    <div className="relative">
      {/* Connection status */}
      <div className="flex items-center justify-between mb-4 p-2 bg-gray-100 rounded">
        <div className="flex items-center gap-2">
          <div
            className={`w-2 h-2 rounded-full ${
              isConnected ? 'bg-green-500' : 'bg-red-500'
            }`}
          />
          <span className="text-sm text-gray-600">
            {isConnected ? 'Connected' : 'Disconnected'}
          </span>
        </div>

        {/* Presence indicators */}
        {presenceUsers.length > 0 && (
          <div className="flex items-center gap-1">
            <span className="text-xs text-gray-500 mr-2">Active users:</span>
            {presenceUsers.slice(0, 5).map((user) => (
              <div
                key={user.id}
                className="w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center text-white text-xs font-semibold"
                title={user.name}
              >
                {user.avatar_url ? (
                  <img
                    src={user.avatar_url}
                    alt={user.name}
                    className="w-full h-full rounded-full"
                  />
                ) : (
                  user.name.charAt(0).toUpperCase()
                )}
              </div>
            ))}
            {presenceUsers.length > 5 && (
              <span className="text-xs text-gray-500">
                +{presenceUsers.length - 5}
              </span>
            )}
          </div>
        )}
      </div>

      {/* Annotated text */}
      <div
        className="p-4 border border-gray-300 rounded bg-white select-text"
        onMouseUp={handleTextSelect}
      >
        {renderAnnotatedText()}
      </div>

      {/* Add annotation button (shown after text selection) */}
      {selectedText && (
        <div className="mt-2">
          <button
            onClick={() => setShowModal(true)}
            className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded text-sm"
          >
            Add Annotation
          </button>
        </div>
      )}

      {/* Modal for creating annotation */}
      {showModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 max-w-md w-full">
            <h3 className="text-lg font-semibold mb-4">Add Annotation</h3>

            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Selected Text
              </label>
              <div className="p-2 bg-yellow-100 rounded text-sm">
                "{selectedText?.text}"
              </div>
            </div>

            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Your Annotation
              </label>
              <textarea
                value={annotationText}
                onChange={(e) => setAnnotationText(e.target.value)}
                className="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                rows={4}
                placeholder="Enter your annotation..."
                autoFocus
              />
            </div>

            <div className="flex justify-end gap-2">
              <button
                onClick={() => {
                  setShowModal(false);
                  setAnnotationText('');
                }}
                className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateAnnotation}
                disabled={!annotationText.trim()}
                className="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded disabled:bg-gray-300 disabled:cursor-not-allowed"
              >
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AnnotationLayer;

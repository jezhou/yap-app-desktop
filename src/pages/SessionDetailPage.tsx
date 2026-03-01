import { useState, useEffect, useCallback } from "react";
import { useParams } from "react-router-dom";
import ConversationList from "../components/conversations/ConversationList";
import UploadDropzone from "../components/upload/UploadDropzone";
import {
  listConversations,
  renameConversation,
  deleteConversation,
} from "../services/conversations";
import { startTranscription } from "../services/transcription";
import type { Conversation } from "../types";

export default function SessionDetailPage() {
  const { sessionId } = useParams<{ sessionId: string }>();
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [error, setError] = useState<string | null>(null);

  const loadConversations = useCallback(async () => {
    if (!sessionId) return;
    try {
      const result = await listConversations(sessionId);
      setConversations(result);
    } catch {
      setError("Failed to load conversations.");
    }
  }, [sessionId]);

  useEffect(() => {
    loadConversations();
  }, [loadConversations]);

  async function handleRename(conversationId: string, title: string) {
    try {
      await renameConversation(conversationId, title);
      await loadConversations();
    } catch {
      setError("Failed to rename conversation.");
    }
  }

  async function handleDelete(conversationId: string) {
    try {
      await deleteConversation(conversationId);
      await loadConversations();
    } catch {
      setError("Failed to delete conversation.");
    }
  }

  if (!sessionId) {
    return <p className="text-error">No session selected.</p>;
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-text">Session</h1>

      {error && (
        <p className="text-sm text-error bg-error/10 rounded-lg px-4 py-2">
          {error}
        </p>
      )}

      <UploadDropzone
        sessionId={sessionId}
        onUploadStart={async (result) => {
          await loadConversations();
          try {
            await startTranscription(result.conversationId);
          } catch {
            setError("Failed to start transcription.");
          }
        }}
        onError={(msg) => setError(msg)}
      />

      {conversations.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-12 border-2 border-dashed border-border rounded-lg">
          <p className="text-text-muted">No conversations yet</p>
          <p className="text-sm text-text-dim mt-1">
            Upload audio to generate insights & transcript
          </p>
        </div>
      ) : (
        <ConversationList
          conversations={conversations}
          sessionId={sessionId}
          onRename={handleRename}
          onDelete={handleDelete}
        />
      )}
    </div>
  );
}

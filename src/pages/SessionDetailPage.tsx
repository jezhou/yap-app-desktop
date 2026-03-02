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
import { onModelDownloadProgress } from "../services/settings";
import type { Conversation } from "../types";

export default function SessionDetailPage() {
  const { sessionId } = useParams<{ sessionId: string }>();
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloadPercent, setDownloadPercent] = useState<number | null>(null);

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

  // Listen for background model download progress
  useEffect(() => {
    const unlisten = onModelDownloadProgress((event) => {
      if (event.percent >= 100) {
        setDownloadPercent(null);
      } else {
        setDownloadPercent(event.percent);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

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

      {downloadPercent !== null && (
        <div className="bg-accent/10 border border-accent/30 rounded-lg px-4 py-3">
          <p className="text-sm text-accent font-medium">
            Downloading transcription model... {downloadPercent}%
          </p>
          <div className="mt-2 h-1.5 bg-surface-hover rounded-full overflow-hidden">
            <div
              className="h-full bg-accent rounded-full transition-all duration-300"
              style={{ width: `${downloadPercent}%` }}
            />
          </div>
        </div>
      )}

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
          } catch (err) {
            const msg =
              err instanceof Error ? err.message : String(err);
            if (msg.includes("ModelNotDownloaded")) {
              setError(
                "Transcription model is still downloading. It will be ready shortly — please try again in a moment.",
              );
            } else {
              setError("Failed to start transcription.");
            }
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

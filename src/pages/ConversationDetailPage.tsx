import { useState, useEffect, useCallback } from "react";
import { useParams } from "react-router-dom";
import TranscriptView from "../components/transcription/TranscriptView";
import InsightsView from "../components/transcription/InsightsView";
import UploadProgress from "../components/upload/UploadProgress";
import { playAudio, pauseAudio, getAudioPosition } from "../services/audio";
import { onTranscriptionProgress } from "../services/transcription";
import { getConversationDetail } from "../services/conversations";
import { updateSpeakerRole } from "../services/conversations";
import type { ConversationDetail, ConversationStatus } from "../types";

type Tab = "transcript" | "insights" | "speakers";

export default function ConversationDetailPage() {
  const { conversationId } = useParams<{
    sessionId: string;
    conversationId: string;
  }>();

  const [detail, setDetail] = useState<ConversationDetail | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("transcript");
  const [playing, setPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [transcriptionPercent, setTranscriptionPercent] = useState(0);
  const [status, setStatus] = useState<ConversationStatus>("uploading");
  const [error, setError] = useState<string | null>(null);

  const loadDetail = useCallback(async () => {
    if (!conversationId) return;
    try {
      const result = await getConversationDetail(conversationId);
      setDetail(result);
      setStatus(result.conversation.status);
    } catch {
      setError("Failed to load conversation.");
    }
  }, [conversationId]);

  useEffect(() => {
    loadDetail();
  }, [loadDetail]);

  // Listen for transcription progress
  useEffect(() => {
    const unlistenPromise = onTranscriptionProgress((event) => {
      if (event.conversationId === conversationId) {
        setTranscriptionPercent(event.percent);
        setStatus("analyzing");
        if (event.percent >= 100) {
          setStatus("completed");
          loadDetail();
        }
      }
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [conversationId, loadDetail]);

  // Poll audio position while playing
  useEffect(() => {
    if (!playing) return;
    const interval = setInterval(async () => {
      try {
        const pos = await getAudioPosition();
        setCurrentTime(pos.positionSeconds);
        setPlaying(pos.playing);
      } catch {
        setPlaying(false);
      }
    }, 250);
    return () => clearInterval(interval);
  }, [playing]);

  async function handlePlayPause() {
    if (!conversationId) return;
    try {
      if (playing) {
        const result = await pauseAudio();
        setCurrentTime(result.positionSeconds);
        setPlaying(false);
      } else {
        const result = await playAudio(conversationId);
        setDuration(result.durationSeconds);
        setPlaying(true);
      }
    } catch {
      setError("Playback error.");
    }
  }

  async function handleSeek(seconds: number) {
    if (!conversationId) return;
    try {
      const result = await playAudio(conversationId, seconds);
      setDuration(result.durationSeconds);
      setCurrentTime(seconds);
      setPlaying(true);
    } catch {
      setError("Seek error.");
    }
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  }

  if (error) {
    return <p className="text-error">{error}</p>;
  }

  if (!detail) {
    return <p className="text-text-muted">Loading...</p>;
  }

  const isProcessing = status === "uploading" || status === "analyzing";

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-text">
          {detail.conversation.title}
        </h1>
      </div>

      {/* Progress indicator for active processing */}
      {isProcessing && (
        <UploadProgress status={status} percent={transcriptionPercent} />
      )}

      {/* Audio player controls */}
      {status === "completed" && (
        <div className="flex items-center gap-4 bg-surface rounded-lg p-3 border border-border">
          <button
            onClick={handlePlayPause}
            className="w-10 h-10 flex items-center justify-center rounded-full bg-accent text-bg hover:bg-accent-hover transition-colors"
          >
            {playing ? "\u23F8" : "\u25B6"}
          </button>
          <div className="flex-1">
            <div
              className="w-full bg-bg rounded-full h-1.5 cursor-pointer"
              onClick={(e) => {
                const rect = e.currentTarget.getBoundingClientRect();
                const ratio = (e.clientX - rect.left) / rect.width;
                handleSeek(ratio * duration);
              }}
            >
              <div
                className="bg-accent h-1.5 rounded-full transition-all"
                style={{
                  width: `${duration > 0 ? (currentTime / duration) * 100 : 0}%`,
                }}
              />
            </div>
          </div>
          <span className="text-xs text-text-muted font-mono whitespace-nowrap">
            {formatTime(currentTime)} / {formatTime(duration)}
          </span>
        </div>
      )}

      {/* Transcript / Insights tabs */}
      {status === "completed" && (
        <>
          <div className="flex gap-1 border-b border-border">
            <button
              onClick={() => setActiveTab("transcript")}
              className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                activeTab === "transcript"
                  ? "border-accent text-accent"
                  : "border-transparent text-text-muted hover:text-text"
              }`}
            >
              Transcript
            </button>
            <button
              onClick={() => setActiveTab("insights")}
              className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                activeTab === "insights"
                  ? "border-accent text-accent"
                  : "border-transparent text-text-muted hover:text-text"
              }`}
            >
              Insights
            </button>
            <button
              onClick={() => setActiveTab("speakers")}
              className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                activeTab === "speakers"
                  ? "border-accent text-accent"
                  : "border-transparent text-text-muted hover:text-text"
              }`}
            >
              Speakers
            </button>
          </div>

          {activeTab === "transcript" && detail.transcription && (
            <TranscriptView
              segments={detail.transcription.segments}
              speakerRoles={detail.speakerRoles}
              onSegmentClick={handleSeek}
              activeTime={currentTime}
            />
          )}

          {activeTab === "insights" && detail.summary && (
            <InsightsView content={detail.summary.content} />
          )}

          {activeTab === "speakers" && (
            <SpeakerRoleEditor
              speakerRoles={detail.speakerRoles}
              conversationId={detail.conversation.id}
              onUpdate={loadDetail}
            />
          )}
        </>
      )}
    </div>
  );
}

function SpeakerRoleEditor({
  speakerRoles,
  conversationId,
  onUpdate,
}: {
  speakerRoles: ConversationDetail["speakerRoles"];
  conversationId: string;
  onUpdate: () => void;
}) {
  const [editingLabel, setEditingLabel] = useState<string | null>(null);
  const [editName, setEditName] = useState("");

  async function handleSave(speakerLabel: string) {
    const trimmed = editName.trim();
    if (!trimmed) return;
    try {
      await updateSpeakerRole(conversationId, speakerLabel, trimmed);
      setEditingLabel(null);
      onUpdate();
    } catch {
      // error handling at page level
    }
  }

  if (speakerRoles.length === 0) {
    return (
      <p className="text-text-muted text-center py-8">
        No speakers detected.
      </p>
    );
  }

  return (
    <div className="space-y-3">
      <p className="text-sm text-text-muted">
        Assign display names to detected speakers.
      </p>
      {speakerRoles.map((role) => (
        <div
          key={role.speaker_label}
          className="flex items-center gap-3 p-3 bg-surface rounded-lg border border-border"
        >
          <span className="text-xs text-text-dim font-mono w-24">
            {role.speaker_label}
          </span>
          {editingLabel === role.speaker_label ? (
            <input
              type="text"
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              onBlur={() => handleSave(role.speaker_label)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleSave(role.speaker_label);
                if (e.key === "Escape") setEditingLabel(null);
              }}
              className="flex-1 bg-bg border border-border rounded px-2 py-1 text-sm text-text focus:outline-none focus:border-accent"
              autoFocus
            />
          ) : (
            <>
              <span className="flex-1 text-sm text-text">
                {role.display_name}
              </span>
              <button
                onClick={() => {
                  setEditingLabel(role.speaker_label);
                  setEditName(role.display_name);
                }}
                className="text-xs text-text-dim hover:text-text transition-colors"
              >
                Edit
              </button>
            </>
          )}
        </div>
      ))}
    </div>
  );
}

import { useState, useEffect, useCallback } from "react";
import { useParams } from "react-router-dom";
import TranscriptView from "../components/transcription/TranscriptView";
import InsightsView from "../components/transcription/InsightsView";
import UploadProgress from "../components/upload/UploadProgress";
import { playAudio, pauseAudio, getAudioPosition } from "../services/audio";
import { onTranscriptionProgress } from "../services/transcription";
import type { ConversationDetail, ConversationStatus } from "../types";
import { invoke } from "@tauri-apps/api/core";

type Tab = "transcript" | "insights";

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
      const result = await invoke<ConversationDetail>(
        "get_conversation_detail",
        { conversationId },
      );
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
        </>
      )}
    </div>
  );
}

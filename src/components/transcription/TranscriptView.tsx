import type { TranscriptionSegment, SpeakerRole } from "../../types";

interface TranscriptViewProps {
  segments: TranscriptionSegment[];
  speakerRoles: SpeakerRole[];
  onSegmentClick?: (startTime: number) => void;
  activeTime?: number;
}

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

function getSpeakerName(
  speaker: string,
  speakerRoles: SpeakerRole[],
): string {
  const role = speakerRoles.find((r) => r.speaker_label === speaker);
  return role?.display_name ?? speaker;
}

function getConfidenceClass(confidence: number): string {
  if (confidence >= 0.9) return "text-text";
  if (confidence >= 0.7) return "text-text-muted";
  return "text-text-dim";
}

export default function TranscriptView({
  segments,
  speakerRoles,
  onSegmentClick,
  activeTime,
}: TranscriptViewProps) {
  if (segments.length === 0) {
    return (
      <p className="text-text-muted text-center py-8">
        No transcript available.
      </p>
    );
  }

  return (
    <div className="space-y-3">
      {segments.map((segment, index) => {
        const isActive =
          activeTime !== undefined &&
          activeTime >= segment.start_time &&
          activeTime < segment.end_time;

        return (
          <div
            key={index}
            className={`flex gap-3 p-3 rounded-lg transition-colors cursor-pointer hover:bg-surface-hover ${
              isActive ? "bg-accent/10 border border-accent/20" : ""
            }`}
            onClick={() => onSegmentClick?.(segment.start_time)}
          >
            <div className="flex-shrink-0 w-16 text-right">
              <span className="text-xs text-text-dim font-mono">
                {formatTime(segment.start_time)}
              </span>
            </div>
            <div className="flex-1 min-w-0">
              <span className="text-xs font-medium text-accent">
                {getSpeakerName(segment.speaker, speakerRoles)}
              </span>
              <p className={`mt-0.5 text-sm ${getConfidenceClass(segment.confidence)}`}>
                {segment.text}
              </p>
            </div>
          </div>
        );
      })}
    </div>
  );
}

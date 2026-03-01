// Data models matching Rust models and tauri-commands.md contracts

export interface Session {
  id: string;
  title: string;
  description: string | null;
  created_at: string;
  updated_at: string;
  conversation_count?: number;
}

export interface Conversation {
  id: string;
  session_id: string;
  sequence_number: number;
  title: string;
  audio_file_path: string;
  duration_seconds: number;
  status: ConversationStatus;
  created_at: string;
  updated_at: string;
}

export type ConversationStatus =
  | "uploading"
  | "analyzing"
  | "completed"
  | "error";

export interface TranscriptionSegment {
  speaker: string;
  text: string;
  start_time: number;
  end_time: number;
  confidence: number;
}

export interface Transcription {
  id: string;
  conversation_id: string;
  full_text: string;
  segments: TranscriptionSegment[];
  created_at: string;
}

export interface Summary {
  id: string;
  conversation_id: string;
  content: string;
  created_at: string;
}

export interface SpeakerRole {
  id: string;
  conversation_id: string;
  speaker_label: string;
  display_name: string;
}

export interface Settings {
  key: string;
  value: string;
}

export interface SearchResult {
  conversation_id: string;
  session_id: string;
  title: string;
  snippet: string;
  rank: number;
}

export interface Model {
  name: string;
  size: string;
  downloaded: boolean;
  quality_tier: string;
}

// IPC command input/output types

export interface UploadAudioInput {
  sessionId: string;
  filePath: string;
}

export interface UploadAudioOutput {
  conversationId: string;
  status: "uploading";
}

export interface PlayAudioInput {
  conversationId: string;
  seekSeconds?: number;
}

export interface PlayAudioOutput {
  playing: true;
  durationSeconds: number;
}

export interface PauseAudioOutput {
  playing: false;
  positionSeconds: number;
}

export interface AudioPositionOutput {
  positionSeconds: number;
  playing: boolean;
}

export interface StartTranscriptionInput {
  conversationId: string;
}

export interface TranscriptionProgressEvent {
  conversationId: string;
  percent: number;
}

export interface ConversationDetail {
  conversation: Conversation;
  transcription: Transcription | null;
  summary: Summary | null;
  speakerRoles: SpeakerRole[];
}

export interface DiarizationStatus {
  segmentationReady: boolean;
  embeddingReady: boolean;
}

export interface ModelDownloadProgressEvent {
  modelName: string;
  percent: number;
}

# Tauri IPC Command Contracts

**Branch**: `001-voice-transcription` | **Date**: 2026-03-01

These are the Rust-side Tauri commands exposed to the frontend
via `invoke()`. Each command is called from React via
`@tauri-apps/api/core`.

## Audio

### upload_audio

Upload an audio file and create a conversation record.

- **Input**: `{ sessionId: string, filePath: string }`
- **Output**: `{ conversationId: string, status: "uploading" }`
- **Errors**: `InvalidFormat`, `FileNotFound`, `DiskSpaceLow`
- **Side effects**: Copies audio to app data dir, creates
  conversation record in SQLite with status `uploading`

### play_audio

Start playback of a conversation's audio file.

- **Input**: `{ conversationId: string, seekSeconds?: number }`
- **Output**: `{ playing: true, durationSeconds: number }`
- **Errors**: `AudioFileNotFound`, `PlaybackError`

### pause_audio

Pause current audio playback.

- **Input**: `{}`
- **Output**: `{ playing: false, positionSeconds: number }`

### get_audio_position

Get current playback position.

- **Input**: `{}`
- **Output**: `{ positionSeconds: number, playing: boolean }`

## Transcription

### start_transcription

Begin transcribing a conversation's audio file.

- **Input**: `{ conversationId: string }`
- **Output**: `{ status: "analyzing" }`
- **Errors**: `ModelNotDownloaded`, `TranscriptionFailed`
- **Events emitted**: `transcription-progress` with
  `{ conversationId, percent: number }`
- **Side effects**: Updates conversation status to `analyzing`.
  On completion, creates transcription + summary records and
  updates status to `completed`. On failure, sets `error`.

### cancel_transcription

Cancel an in-progress transcription.

- **Input**: `{ conversationId: string }`
- **Output**: `{ status: "error", reason: "cancelled" }`

## Sessions & Conversations

### list_sessions

List all sessions, ordered by most recent.

- **Input**: `{ limit?: number, offset?: number }`
- **Output**: `Session[]` — id, title, description, created_at,
  conversation_count

### create_session

Create a new session.

- **Input**: `{ title: string, description?: string }`
- **Output**: `{ sessionId: string }`

### update_session

Rename or update a session.

- **Input**: `{ sessionId: string, title?: string,
  description?: string }`
- **Output**: `{ updated: true }`

### delete_session

Delete a session and all its conversations.

- **Input**: `{ sessionId: string }`
- **Output**: `{ deleted: true }`
- **Side effects**: Deletes all conversations, transcriptions,
  summaries, speaker roles, and audio files in the session.

### list_conversations

List conversations in a session.

- **Input**: `{ sessionId: string }`
- **Output**: `Conversation[]` — id, sequence_number, title,
  duration_seconds, status, created_at

### get_conversation_detail

Get full conversation with transcription and summary.

- **Input**: `{ conversationId: string }`
- **Output**: `{ conversation, transcription, summary,
  speakerRoles }`

### rename_conversation

- **Input**: `{ conversationId: string, title: string }`
- **Output**: `{ updated: true }`

### delete_conversation

- **Input**: `{ conversationId: string }`
- **Output**: `{ deleted: true }`
- **Side effects**: Deletes audio file, transcription, summary,
  speaker roles.

## Speaker Roles

### update_speaker_role

Assign a display name to a detected speaker.

- **Input**: `{ conversationId: string, speakerLabel: string,
  displayName: string }`
- **Output**: `{ updated: true }`

## Search

### search_conversations

Full-text search across all sessions, conversations,
transcriptions, and summaries.

- **Input**: `{ query: string, limit?: number }`
- **Output**: `SearchResult[]` — conversationId, sessionId,
  title, snippet, rank

## Export

### export_markdown

Export a conversation to Markdown file.

- **Input**: `{ conversationId: string, outputPath: string }`
- **Output**: `{ exported: true, path: string }`

### export_pdf

Export a conversation to PDF file.

- **Input**: `{ conversationId: string, outputPath: string }`
- **Output**: `{ exported: true, path: string }`

## Settings

### get_settings

- **Input**: `{}`
- **Output**: `Settings` — selectedModel, uiPreferences, etc.

### update_settings

- **Input**: `{ key: string, value: any }`
- **Output**: `{ updated: true }`

## Local Model Management

Models are managed via sherpa-rs (sherpa-onnx). Two categories:
- **STT models**: Whisper ONNX variants (base, small, medium)
- **Diarization models**: Segmentation (pyannote) + speaker
  embedding (3dspeaker) — small, bundled by default

### list_available_models

List STT models available for download.

- **Input**: `{}`
- **Output**: `Model[]` — name, size, downloaded, quality_tier

### download_model

Download a sherpa-onnx STT model for local use.

- **Input**: `{ modelName: string }`
- **Output**: streams progress events
- **Events emitted**: `model-download-progress` with
  `{ modelName, percent: number }`

### delete_model

Delete a downloaded local model.

- **Input**: `{ modelName: string }`
- **Output**: `{ deleted: true }`

### get_diarization_status

Check if diarization models are available locally.

- **Input**: `{}`
- **Output**: `{ segmentationReady: boolean,
  embeddingReady: boolean }`

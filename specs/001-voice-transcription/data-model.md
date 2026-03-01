# Data Model: Voice Conversation Transcription

**Branch**: `001-voice-transcription` | **Date**: 2026-03-01

## Storage

- **Engine**: SQLite via `tauri-plugin-sql` with FTS5
- **Location**: App data directory (platform-specific)
- **Audio files**: Stored on filesystem alongside database;
  paths referenced in `conversations` table

## Entities

### sessions

| Field       | Type     | Constraints                    |
|-------------|----------|--------------------------------|
| id          | TEXT     | PK, UUID                       |
| title       | TEXT     | NOT NULL, user-editable         |
| description | TEXT     | nullable                        |
| created_at  | TEXT     | ISO 8601 datetime, NOT NULL     |
| updated_at  | TEXT     | ISO 8601 datetime, NOT NULL     |

### conversations

| Field            | Type     | Constraints                    |
|------------------|----------|--------------------------------|
| id               | TEXT     | PK, UUID                       |
| session_id       | TEXT     | FK → sessions.id, NOT NULL     |
| sequence_number  | INTEGER  | auto-increment within session  |
| title            | TEXT     | NOT NULL, defaults to filename |
| audio_file_path  | TEXT     | NOT NULL, relative to app data |
| duration_seconds | REAL     | NOT NULL                       |
| status           | TEXT     | NOT NULL, enum (see below)     |
| created_at       | TEXT     | ISO 8601 datetime, NOT NULL    |
| updated_at       | TEXT     | ISO 8601 datetime, NOT NULL    |

**Status enum**: `uploading` → `analyzing` → `completed` | `error`

### transcriptions

| Field           | Type     | Constraints                     |
|-----------------|----------|---------------------------------|
| id              | TEXT     | PK, UUID                        |
| conversation_id | TEXT     | FK → conversations.id, UNIQUE   |
| full_text       | TEXT     | NOT NULL                        |
| segments        | TEXT     | JSON array (see below)          |
| created_at      | TEXT     | ISO 8601 datetime, NOT NULL     |

**Segment JSON schema**:
```json
{
  "speaker": "Person 1",
  "text": "Hello, how are you?",
  "start_time": 0.0,
  "end_time": 2.5,
  "confidence": 0.95
}
```

### speaker_roles

| Field           | Type     | Constraints                     |
|-----------------|----------|---------------------------------|
| id              | TEXT     | PK, UUID                        |
| conversation_id | TEXT     | FK → conversations.id           |
| speaker_label   | TEXT     | NOT NULL (e.g., "Speaker 1")    |
| display_name    | TEXT     | NOT NULL, user-editable          |

### summaries

| Field           | Type     | Constraints                     |
|-----------------|----------|---------------------------------|
| id              | TEXT     | PK, UUID                        |
| conversation_id | TEXT     | FK → conversations.id, UNIQUE   |
| content         | TEXT     | NOT NULL, bullet-point key points |
| created_at      | TEXT     | ISO 8601 datetime, NOT NULL     |

### settings

| Field | Type | Constraints              |
|-------|------|--------------------------|
| key   | TEXT | PK                       |
| value | TEXT | NOT NULL, JSON-encoded   |

Used for: selected model, model preferences, UI preferences.
No API keys needed — all processing is local.

### transcription_fts (FTS5 virtual table)

External content FTS5 table indexing:
- `sessions.title`
- `conversations.title`
- `transcriptions.full_text`
- `summaries.content`

Supports FR-008 search across all text content.

## Relationships

```
sessions 1──* conversations 1──1 transcriptions
                             1──1 summaries
                             1──* speaker_roles
```

## State Transitions

```
Conversation status:
  uploading → analyzing → completed
  uploading → error
  analyzing → error
  error → analyzing (retry)
```

## Validation Rules

- Session title: 1-200 characters
- Conversation title: 1-200 characters
- Audio file: must exist on disk, supported format
  (.mp3, .wav, .m4a, .ogg, .webm)
- Speaker display_name: 1-100 characters
- Duration: 0 < duration ≤ soft limit warn at 14400s (4 hours)

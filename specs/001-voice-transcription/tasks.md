# Tasks: Voice Conversation Transcription

**Input**: Design documents from `/specs/001-voice-transcription/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/tauri-commands.md, research.md, quickstart.md

**Tests**: Not explicitly requested in spec. Test tasks are omitted. Add them if TDD is desired.

**Organization**: Tasks grouped by user story (P1, P2, P3) for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story (US1, US2, US3)
- Exact file paths included in every task

## Path Conventions

- **Frontend**: `src/` (React + TypeScript)
- **Backend**: `src-tauri/src/` (Rust + Tauri)
- **Tests**: `tests/` (frontend), `src-tauri/` for `cargo test`

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Scaffold Tauri v2 project with all dependencies and tooling configured.

- [X] T001 Initialize Tauri v2 project with React 19, Vite 6, and TypeScript 5.x using `pnpm create tauri-app` in repository root
- [X] T002 [P] Add Rust dependencies to src-tauri/Cargo.toml: sherpa-rs, tauri-plugin-sql (sqlite), rodio, symphonia, uuid, serde/serde_json, chrono, tokio
- [X] T003 [P] Add frontend dependencies to package.json: tailwindcss 4, @tailwindcss/vite, react-router-dom, pdfmake, @tauri-apps/api, @tauri-apps/plugin-dialog, @tauri-apps/plugin-fs
- [X] T004 [P] Configure Tailwind CSS 4 with dark theme defaults and green accent palette (#00C853) in src/index.css

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Database, data models, type definitions, routing, and app shell that ALL user stories depend on.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T005 Create SQLite database module with connection management and migration runner in src-tauri/src/db/mod.rs
- [X] T006 Create database migrations for all tables (sessions, conversations, transcriptions, summaries, speaker_roles, settings) and FTS5 virtual table in src-tauri/src/db/migrations/
- [X] T007 [P] Define Rust data models with serde Serialize/Deserialize for Session, Conversation, Transcription, Summary, SpeakerRole, Settings, and TranscriptionSegment in src-tauri/src/models/ (session.rs, conversation.rs, transcription.rs, summary.rs)
- [X] T008 [P] Define shared TypeScript types and interfaces matching all IPC contracts in src/types/index.ts (Session, Conversation, Transcription, Summary, SpeakerRole, Settings, SearchResult, TranscriptionSegment)
- [X] T009 Create Tauri command registration in src-tauri/src/main.rs with all command modules imported and registered via tauri::generate_handler!
- [X] T010 [P] Implement get_settings and update_settings commands (key-value store for app config) in src-tauri/src/commands/settings.rs
- [X] T011 [P] Configure frontend routing with React Router in src/App.tsx: routes for sessions list (/), session detail (/:sessionId), conversation detail (/:sessionId/:conversationId), settings (/settings), setup wizard (/setup)
- [X] T012 [P] Create app layout shell with collapsible left sidebar and main content area in src/components/layout/ (Sidebar.tsx, MainContent.tsx, NavBar.tsx)

**Checkpoint**: Foundation ready — user story implementation can now begin.

---

## Phase 3: User Story 1 — Upload and Transcribe Audio (Priority: P1) MVP

**Goal**: User uploads an audio file, the app transcribes it locally via sherpa-rs, generates a summary, and displays the result with timestamped speaker-labeled segments. Includes setup wizard for first-run model download and audio playback.

**Independent Test**: Upload a short audio file (.mp3 or .wav), verify transcription with summary appears, play back audio, confirm fully offline operation.

### Rust Backend — US1

- [X] T013 [P] [US1] Implement audio format validation (accept .mp3, .wav, .m4a, .ogg, .webm) and conversion to 16kHz mono WAV via symphonia in src-tauri/src/services/audio_player.rs
- [X] T014 [P] [US1] Implement create_session command (create a session record in SQLite, return sessionId) in src-tauri/src/commands/sessions.rs
- [X] T015 [US1] Implement upload_audio command (validate format, copy file to app data dir, create conversation record with status "uploading") in src-tauri/src/commands/audio.rs
- [X] T016 [US1] Implement sherpa-rs transcription service: load Whisper ONNX model, run STT, run speaker diarization (pyannote segmentation + 3dspeaker embedding), merge into timestamped speaker-labeled segments in src-tauri/src/services/transcription.rs
- [X] T017 [US1] Implement summarizer service to extract 1-3 sentence summary and bullet-point key insights from transcription text in src-tauri/src/services/summarizer.rs
- [X] T018 [US1] Implement start_transcription command (orchestrate: convert audio, run STT + diarization, generate summary, emit transcription-progress events, update conversation status) and cancel_transcription command in src-tauri/src/commands/transcription.rs
- [X] T019 [P] [US1] Implement audio playback commands (play_audio with seek, pause_audio, get_audio_position) using rodio/symphonia in src-tauri/src/commands/audio.rs
- [X] T020 [P] [US1] Implement model management commands (list_available_models, download_model with progress events, delete_model, get_diarization_status) in src-tauri/src/commands/settings.rs

### Frontend Services — US1

- [X] T021 [P] [US1] Create audio service (IPC wrappers for upload_audio, play_audio, pause_audio, get_audio_position) in src/services/audio.ts
- [X] T022 [P] [US1] Create transcription service (IPC wrappers for start_transcription, cancel_transcription, listen to transcription-progress events) in src/services/transcription.ts
- [X] T023 [P] [US1] Create settings service (IPC wrappers for get_settings, update_settings, list_available_models, download_model, delete_model, get_diarization_status) in src/services/settings.ts

### Frontend Components — US1

- [X] T024 [US1] Create SetupWizard component: model selection (base/small/medium with size and RAM info), download progress bar, diarization model status, "Get Started" completion in src/components/settings/SetupWizard.tsx
- [X] T025 [US1] Create UploadDropzone component with drag-and-drop support (onDrop handler), file picker via Tauri dialog, supported format display, and session creation flow in src/components/upload/UploadDropzone.tsx
- [X] T026 [P] [US1] Create UploadProgress component showing transcription progress percentage and status (uploading/analyzing/completed/error) in src/components/upload/UploadProgress.tsx
- [X] T027 [US1] Create TranscriptView component: render timestamped segments with speaker labels, confidence indicators, and scroll-to-segment on audio playback in src/components/transcription/TranscriptView.tsx
- [X] T028 [P] [US1] Create InsightsView component: render summary sentence and bullet-point key insights in src/components/transcription/InsightsView.tsx
- [X] T029 [US1] Create ConversationDetailPage with Transcript/Insights toggle tabs, audio player controls (play/pause/seek), and progress indicator in src/pages/ConversationDetailPage.tsx
- [X] T030 [US1] Create SettingsPage with model management UI (list models, download/delete, current selection, diarization status) in src/pages/SettingsPage.tsx

**Checkpoint**: US1 complete — user can download a model, upload audio, see transcription + summary, and play back audio. This is a fully functional MVP.

---

## Phase 4: User Story 2 — Browse and Manage Conversations (Priority: P2)

**Goal**: User sees all sessions and conversations in a sidebar, can select, rename, delete, search across all text content, and assign speaker names.

**Independent Test**: Upload several audio files across multiple sessions, verify sidebar shows all sessions/conversations, search by transcript text, rename and delete items, assign speaker names.

### Rust Backend — US2

- [X] T031 [P] [US2] Implement session CRUD commands (list_sessions with pagination, update_session, delete_session with cascade delete of all child records and audio files) in src-tauri/src/commands/sessions.rs
- [X] T032 [P] [US2] Implement conversation query commands (list_conversations for a session, get_conversation_detail with transcription + summary + speaker_roles, rename_conversation, delete_conversation with cascade) in src-tauri/src/commands/conversations.rs
- [X] T033 [P] [US2] Implement update_speaker_role command (upsert speaker_label → display_name mapping for a conversation) in src-tauri/src/commands/conversations.rs
- [X] T034 [US2] Implement search_conversations FTS5 query command (search across sessions.title, conversations.title, transcriptions.full_text, summaries.content; return ranked results with snippets) in src-tauri/src/commands/search.rs

### Frontend Services — US2

- [X] T035 [P] [US2] Create sessions service (IPC wrappers for list_sessions, create_session, update_session, delete_session) in src/services/sessions.ts
- [X] T036 [P] [US2] Create conversations service (IPC wrappers for list_conversations, get_conversation_detail, rename_conversation, delete_conversation, update_speaker_role) in src/services/conversations.ts
- [X] T037 [P] [US2] Create search service (IPC wrapper for search_conversations) in src/services/search.ts

### Frontend Components — US2

- [X] T038 [US2] Create SessionList sidebar component showing all sessions ordered by most recent, with session title and conversation count in src/components/sessions/SessionList.tsx
- [X] T039 [P] [US2] Create SessionCard component (session title, description, conversation count, created date, rename/delete actions) in src/components/sessions/SessionCard.tsx
- [X] T040 [US2] Create SessionsPage displaying session list with "New Session" action in src/pages/SessionsPage.tsx
- [X] T041 [US2] Create SessionDetailPage with conversation table (ID, title/summary preview, status badge, date, duration, action icons) in src/pages/SessionDetailPage.tsx
- [X] T042 [P] [US2] Create ConversationList component and ConversationRow component for the session detail table in src/components/conversations/ConversationList.tsx and src/components/conversations/ConversationRow.tsx
- [X] T043 [US2] Create SearchBar component with debounced FTS5 search, result dropdown with conversation snippets, and navigation to result in src/components/shared/SearchBar.tsx
- [X] T044 [US2] Add inline rename (editable title) and delete (with confirmation dialog) actions for sessions and conversations in sidebar and detail views
- [X] T045 [US2] Add speaker role assignment UI: editable speaker name fields in "Assign Roles" section of ConversationDetailPage in src/pages/ConversationDetailPage.tsx

**Checkpoint**: US1 + US2 complete — user can browse, search, organize, rename, delete, and manage all conversations and sessions.

---

## Phase 5: User Story 3 — Export Conversations (Priority: P3)

**Goal**: User exports a conversation (title, summary, full transcription with speaker labels) as a Markdown file or a PDF file via native save dialog.

**Independent Test**: Export a completed conversation to Markdown and PDF, open both in standard viewers, verify they contain title, summary, and full transcript with speaker labels.

### Backend + Services — US3

- [X] T046 [P] [US3] Implement Markdown export formatter (title, summary, timestamped speaker-labeled segments) in src-tauri/src/services/exporter.rs
- [X] T047 [US3] Implement export_markdown command (gather conversation data, format via exporter service, write .md file to outputPath) in src-tauri/src/commands/export.rs
- [X] T048 [US3] Implement export_pdf command (gather conversation data, return structured payload for frontend PDF generation via pdfmake) in src-tauri/src/commands/export.rs

### Frontend — US3

- [X] T049 [P] [US3] Create frontend export service: IPC wrapper for export_markdown, PDF generation via pdfmake (lazy-loaded), file write via Tauri fs API in src/services/export.ts
- [X] T050 [US3] Create ExportDialog component with format picker (Markdown/PDF), native file save dialog via @tauri-apps/plugin-dialog, and export progress indicator in src/components/export/ExportDialog.tsx
- [X] T051 [US3] Wire Export button into ConversationDetailPage toolbar; disable when conversation status is not "completed" in src/pages/ConversationDetailPage.tsx

**Checkpoint**: All three user stories complete — full upload → transcribe → browse → export workflow functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, UX polish, and validation across all stories.

- [X] T052 [P] Add edge case error handling: unsupported format error with supported formats list, 4+ hour file duration warning, disk space check before processing, "no speech detected" notification, duplicate file handling (always create new conversation) in src-tauri/src/commands/audio.rs and src-tauri/src/commands/transcription.rs
- [X] T053 [P] Create StatusBadge component for conversation states (green "Completed", yellow "Analyzing", red "Error") in src/components/shared/StatusBadge.tsx
- [X] T054 [P] Create breadcrumb navigation component (Home / Session / Audio path) in src/components/layout/NavBar.tsx
- [X] T055 Implement empty state UIs: dashed border "Upload audio to generate insights & transcript" prompt on session detail when no conversations exist, "Create your first session" on sessions page in src/pages/SessionsPage.tsx and src/pages/SessionDetailPage.tsx
- [X] T056 Run quickstart.md end-to-end validation: first launch → setup wizard → create session → upload audio → transcription completes → browse in sidebar → search → export to Markdown and PDF

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 completion
- **US2 (Phase 4)**: Depends on Phase 2 completion; integrates with US1 components but independently testable
- **US3 (Phase 5)**: Depends on Phase 2 completion; uses data created by US1/US2 but independently testable
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2. No dependencies on US2 or US3. Delivers MVP alone.
- **US2 (P2)**: Can start after Phase 2. Extends US1's create_session to full CRUD. Independently testable with any existing conversations.
- **US3 (P3)**: Can start after Phase 2. Only needs a completed conversation to export. Independently testable.

### Within Each User Story

1. Rust services before Rust commands (services contain business logic; commands are thin wrappers)
2. Rust commands before frontend services (frontend IPC wrappers call Rust commands)
3. Frontend services before frontend components (components consume services)
4. Shared/leaf components before page components (pages compose components)

### Parallel Opportunities

**Phase 1**: T002, T003, T004 can all run in parallel (after T001)

**Phase 2**: T007, T008, T010, T011, T012 can all run in parallel (after T005, T006)

**Phase 3 (US1)**: T013, T014, T015, T019, T020 can run in parallel (different Rust modules). T021, T022, T023 can run in parallel (different .ts files). T026, T028 can run in parallel with other components.

**Phase 4 (US2)**: T031, T032, T033 can run in parallel (Rust commands). T035, T036, T037 can run in parallel (frontend services). T039, T042 can run in parallel (leaf components).

**Phase 5 (US3)**: T046, T049 can run in parallel (Rust exporter + frontend export service).

**Phase 6**: T052, T053, T054 can all run in parallel.

---

## Parallel Example: User Story 1

```bash
# Launch Rust backend tasks in parallel (different .rs files):
Task: "Implement audio format validation and conversion in src-tauri/src/services/audio_player.rs"
Task: "Implement create_session command in src-tauri/src/commands/sessions.rs"
Task: "Implement audio playback commands in src-tauri/src/commands/audio.rs"
Task: "Implement model management commands in src-tauri/src/commands/settings.rs"

# After services complete, launch frontend services in parallel:
Task: "Create audio service in src/services/audio.ts"
Task: "Create transcription service in src/services/transcription.ts"
Task: "Create settings service in src/services/settings.ts"

# Launch independent leaf components in parallel:
Task: "Create UploadProgress component in src/components/upload/UploadProgress.tsx"
Task: "Create InsightsView component in src/components/transcription/InsightsView.tsx"
```

---

## Parallel Example: User Story 2

```bash
# Launch Rust CRUD commands in parallel (different command groups):
Task: "Implement session CRUD commands in src-tauri/src/commands/sessions.rs"
Task: "Implement conversation query commands in src-tauri/src/commands/conversations.rs"
Task: "Implement update_speaker_role command in src-tauri/src/commands/conversations.rs"

# Launch frontend services in parallel:
Task: "Create sessions service in src/services/sessions.ts"
Task: "Create conversations service in src/services/conversations.ts"
Task: "Create search service in src/services/search.ts"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T012) — CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T013-T030)
4. **STOP and VALIDATE**: Upload a .wav file, verify transcription + summary + playback work
5. Deploy/demo if ready — this is a usable standalone transcription tool

### Incremental Delivery

1. Setup + Foundational (T001-T012) → Foundation ready
2. User Story 1 (T013-T030) → Test independently → **MVP!** (upload, transcribe, view, playback)
3. User Story 2 (T031-T045) → Test independently → Browse, search, manage conversations
4. User Story 3 (T046-T051) → Test independently → Export to Markdown + PDF
5. Polish (T052-T056) → Edge cases, status badges, empty states, breadcrumbs
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T012)
2. Once Foundational is done:
   - Developer A: User Story 1 (core upload + transcription flow)
   - Developer B: User Story 2 (browse + manage, after minimal US1 backend is available)
   - Developer C: User Story 3 (export, after at least one conversation exists)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks in same phase
- [Story] label maps task to specific user story for traceability
- All processing is local-only (privacy-first, no cloud). No API keys needed.
- PDF export uses pdfmake (JS, lazy-loaded) on the frontend; Markdown export is Rust-side
- Audio playback uses Rust-side rodio/symphonia for cross-platform format consistency
- sherpa-rs provides both STT (Whisper ONNX) and diarization (pyannote + 3dspeaker) in one crate
- Commit after each task or logical group
- Stop at any checkpoint to validate the story independently

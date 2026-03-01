# Implementation Plan: Voice Conversation Transcription

**Branch**: `001-voice-transcription` | **Date**: 2026-03-01 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-voice-transcription/spec.md`

## Summary

Build a cross-platform desktop app (Yap) using Tauri v2 that
lets users upload audio files, transcribe them locally via
sherpa-onnx (Whisper ONNX models + speaker diarization), generate
summaries/insights, organize conversations in sessions with a
ChatGPT-style sidebar, and export to Markdown or PDF. Fully
offline, privacy-first. Dark-themed UI inspired by Figma designs.

## Technical Context

**Language/Version**: Rust (latest stable) + TypeScript 5.x
**Primary Dependencies**: Tauri v2, React 19, Vite 6, Tailwind
CSS 4, sherpa-rs (local STT + diarization), rodio/symphonia,
pdfmake
**Storage**: SQLite via tauri-plugin-sql with FTS5 (metadata +
search), filesystem (audio files + downloaded models)
**Testing**: Vitest + @tauri-apps/api/mocks (frontend),
cargo test (Rust), WebdriverIO via tauri-driver (E2E)
**Target Platform**: macOS, Windows, Linux (desktop)
**Project Type**: desktop-app
**Performance Goals**: <2s cold start, <150 MB idle memory,
transcription within 2x audio duration (local)
**Constraints**: Fully offline, <150 MB idle memory,
privacy-first (all data local, no cloud)
**Scale/Scope**: Single user, hundreds of conversations,
~4 screens (setup wizard, sessions list, conversation detail,
settings)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after
Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Privacy-First | PASS | All data local. No cloud backends. No API keys. No telemetry. Fully offline operation. |
| II. Lightweight by Design | PASS | Tauri v2: ~5-10 MB bundle, ~20-40 MB idle memory, <500ms startup. All within constitution limits. |
| III. Cross-Platform | PASS | Tauri builds for macOS, Windows, Linux. CI matrix on all three. Rust-side audio for format consistency. |
| IV. Test-Driven Quality | PASS | Vitest (frontend), cargo test (Rust), WebdriverIO (E2E). CI on all platforms. |
| V. Simplicity | PASS | Single codebase, standard patterns, single transcription backend (sherpa-rs), no cloud abstraction layer. |

**Post-Phase 1 re-check**: All gates still pass. No violations
requiring complexity justification.

## Project Structure

### Documentation (this feature)

```text
specs/001-voice-transcription/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── tauri-commands.md
└── tasks.md               # (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/                        # React frontend (TypeScript)
├── components/
│   ├── layout/             # Sidebar, NavBar, MainContent
│   ├── sessions/           # SessionList, SessionCard
│   ├── conversations/      # ConversationList, ConversationRow
│   ├── transcription/      # TranscriptView, InsightsView
│   ├── upload/             # UploadDropzone, UploadProgress
│   ├── export/             # ExportDialog
│   ├── settings/           # SettingsPage, SetupWizard
│   └── shared/             # Button, SearchBar, StatusBadge
├── pages/
│   ├── SessionsPage.tsx
│   ├── SessionDetailPage.tsx
│   ├── ConversationDetailPage.tsx
│   └── SettingsPage.tsx
├── services/               # Tauri IPC wrappers
│   ├── audio.ts
│   ├── transcription.ts
│   ├── sessions.ts
│   ├── conversations.ts
│   ├── search.ts
│   ├── export.ts
│   └── settings.ts
├── hooks/
├── stores/
├── types/
├── App.tsx
└── main.tsx

src-tauri/                  # Rust backend
├── src/
│   ├── main.rs
│   ├── commands/           # Tauri command handlers
│   │   ├── audio.rs
│   │   ├── transcription.rs
│   │   ├── sessions.rs
│   │   ├── conversations.rs
│   │   ├── search.rs
│   │   ├── export.rs
│   │   └── settings.rs
│   ├── services/           # Business logic
│   │   ├── audio_player.rs
│   │   ├── transcription.rs # sherpa-rs STT + diarization
│   │   ├── summarizer.rs
│   │   └── exporter.rs
│   ├── models/
│   │   ├── session.rs
│   │   ├── conversation.rs
│   │   ├── transcription.rs
│   │   └── summary.rs
│   └── db/
│       ├── mod.rs
│       └── migrations/
├── Cargo.toml
└── tauri.conf.json

tests/                      # Frontend tests
├── unit/
└── integration/
```

**Structure Decision**: Single Tauri project with standard
`src/` (frontend) + `src-tauri/` (Rust backend) layout. This is
Tauri's default structure and the simplest viable approach for a
desktop app with no separate server component.

## Complexity Tracking

No constitution violations. No complexity justification needed.

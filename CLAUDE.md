# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is Yap?

Yap is a cross-platform desktop app for local voice conversation transcription. Users upload audio files, which are transcribed and diarized entirely on-device using sherpa-onnx (via sherpa-rs). No cloud services, no API keys, fully offline.

## Tech Stack

- **Backend**: Rust (stable) via Tauri v2 — audio processing, transcription, storage
- **Frontend**: React 19 + TypeScript 5.x + Vite 6 + Tailwind CSS 4
- **Transcription**: sherpa-rs (Whisper ONNX for STT, pyannote + 3dspeaker for diarization)
- **Storage**: SQLite via tauri-plugin-sql with FTS5 full-text search
- **Audio**: rodio/symphonia (Rust-side playback and format conversion)
- **PDF export**: pdfmake (frontend, lazy-loaded)

## Commands

```bash
pnpm install            # Install frontend dependencies
pnpm tauri dev          # Run app in dev mode (frontend + Tauri)
pnpm dev                # Frontend only (no Tauri)
pnpm test               # Frontend tests (Vitest)
pnpm test -- path/to/test  # Run a single frontend test
cargo test              # Rust backend tests (run from src-tauri/)
cargo clippy            # Rust linting (run from src-tauri/)
cargo fmt               # Rust formatting (run from src-tauri/)
pnpm tauri build        # Production build
```

## Prerequisites

- Rust (latest stable via rustup), Node.js 20+, pnpm
- **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `libssl-dev`
- **macOS**: Xcode Command Line Tools
- **Windows**: Visual Studio Build Tools, WebView2

## Architecture

```
src/                        # React frontend (TypeScript)
├── components/             # UI components (layout/, sessions/, transcription/, upload/, export/, settings/, shared/)
├── pages/                  # Route pages (SessionsPage, SessionDetailPage, ConversationDetailPage, SettingsPage)
├── services/               # Tauri IPC wrappers (each .ts file wraps a group of Rust commands)
├── hooks/                  # React hooks
├── stores/                 # State management
└── types/                  # Shared TypeScript types matching Rust models

src-tauri/                  # Rust backend
├── src/
│   ├── main.rs             # Tauri app entry, command registration via generate_handler!
│   ├── commands/           # Tauri IPC command handlers (thin wrappers calling services)
│   ├── services/           # Business logic (transcription, audio, summarizer, exporter)
│   ├── models/             # Serde data models (Session, Conversation, Transcription, Summary, SpeakerRole)
│   └── db/                 # SQLite connection management and migrations
├── Cargo.toml
└── tauri.conf.json
```

### Key Patterns

- **Frontend ↔ Rust**: All communication via Tauri IPC commands (invoke/events). Contracts defined in `specs/001-voice-transcription/contracts/tauri-commands.md`.
- **Command → Service layering**: Rust `commands/` are thin handlers; business logic lives in `services/`. Services before commands when implementing.
- **Data flow**: Frontend services → Tauri invoke → Rust commands → Rust services → SQLite/filesystem
- **Progress events**: Long operations (transcription, model download) emit Tauri events that the frontend listens to.
- **Audio files**: Stored on filesystem in app data dir; paths referenced in SQLite `conversations` table.

### Data Model

Six SQLite tables: `sessions` (groups of conversations), `conversations` (uploaded audio files), `transcriptions` (STT output with JSON segments), `summaries` (key points), `speaker_roles` (user-assigned speaker names), `settings` (key-value config). FTS5 virtual table indexes across sessions, conversations, transcriptions, and summaries for search.

Relationships: `sessions 1──* conversations 1──1 transcriptions, 1──1 summaries, 1──* speaker_roles`

## Constitution (Governing Principles)

These principles are non-negotiable and override ad-hoc decisions:

1. **Privacy-First**: All data local. No telemetry, no network calls for core features. Fully offline.
2. **Lightweight**: <150 MB idle memory, <2s cold start, <1% background CPU. Justify every dependency.
3. **Cross-Platform**: macOS, Windows, Linux from single codebase. CI on all three.
4. **Test-Driven Quality**: Tests required for every feature. Green CI gates merges.
5. **Simplicity**: YAGNI. No premature abstractions. Justify complexity in writing.

Full constitution: `.specify/memory/constitution.md`

## Code Style

- **Rust**: `cargo fmt` + `cargo clippy` — standard Rust conventions
- **TypeScript/React**: Standard React conventions
- **Commits**: Conventional commits (e.g., `feat:`, `fix:`, `refactor:`)
- **UI theme**: Dark background, light text, green accent (#00C853)

## Feature Specs (Speckit)

Features are specified in `specs/<feature-id>/` with these artifacts:
- `spec.md` — requirements and acceptance scenarios
- `plan.md` — architecture and implementation plan
- `tasks.md` — ordered implementation tasks
- `data-model.md` — entity definitions and relationships
- `contracts/` — IPC command contracts
- `quickstart.md` — setup and first-run guide

Use `/speckit.*` slash commands to manage specs (specify, plan, tasks, implement, analyze, clarify).

<!-- MANUAL ADDITIONS START -->

## Implementation Workflow

When implementing a feature, follow this process:

1. **Specify**: Run `/speckit.specify` to create or update the feature spec
2. **Plan**: Run `/speckit.plan` to generate the implementation plan
3. **Tasks**: Run `/speckit.tasks` to generate ordered tasks
4. **Implement**: Spawn a team of agents following the personas and interaction protocols in `.claude/team-personas.md`

### Agent Team Structure

The orchestrating agent acts as **Maya (PM)** — creating tasks from `tasks.md`, assigning work, gating phases, and enforcing the spec. Six agents are spawned:

| Agent | Persona | Focus |
|-------|---------|-------|
| `architect` | Kai | Spec compliance, contract validation, constitution checks |
| `dev-rust` | Russ | All `src-tauri/` — models, services, commands, migrations |
| `dev-frontend` | Tess | All `src/` — components, pages, services, hooks, types |
| `qa-functional` | Val | Acceptance tests, edge case coverage, spec scenario validation |
| `qa-integration` | Seb | Data integrity, cascade ops, state transitions, cross-story flows |
| `designer` | Ren | Visual consistency, dark theme, accessibility, design reference alignment |

### Pipeline

Every task flows: **Developer → Architect review → QA tests → Designer review (if UI)**. No task is complete until all applicable reviewers sign off. Developers work in parallel lanes (backend/frontend). Phase gates block advancement until checkpoints pass.

Full personas and interaction protocols: `.claude/team-personas.md`

<!-- MANUAL ADDITIONS END -->

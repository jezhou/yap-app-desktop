# Quickstart: Yap Voice Transcription

**Branch**: `001-voice-transcription` | **Date**: 2026-03-01

## Prerequisites

- Rust (latest stable via rustup)
- Node.js 20+ and pnpm
- Platform-specific Tauri v2 dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools, WebView2
  - **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`,
    `librsvg2-dev`, `libssl-dev`

## Setup

```bash
# Clone and install
git clone <repo-url>
cd yap-app-desktop
pnpm install

# Start dev server (frontend + Tauri)
pnpm tauri dev
```

## Project Structure

```text
yap-app-desktop/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── pages/              # Route pages
│   ├── services/           # Frontend service layer (IPC calls)
│   ├── hooks/              # React hooks
│   ├── stores/             # State management
│   └── App.tsx             # Root component
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Tauri app entry
│   │   ├── commands/       # Tauri IPC commands
│   │   ├── services/       # Business logic
│   │   ├── models/         # Data models
│   │   └── transcription/  # Transcription backends
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/                  # Frontend tests
│   ├── unit/
│   └── integration/
├── specs/                  # Feature specifications
└── package.json
```

## Key Commands

```bash
# Development
pnpm tauri dev          # Run app in dev mode
pnpm dev                # Frontend only (no Tauri)

# Testing
pnpm test               # Frontend unit tests (Vitest)
cd src-tauri && cargo test  # Rust backend tests

# Building
pnpm tauri build        # Production build for current platform
```

## First Run

1. Launch the app → setup wizard appears
2. Select a Whisper model to download:
   - **Base** (142 MB, ~1 GB RAM) — fastest, good for clear audio
   - **Small** (466 MB, ~2 GB RAM) — recommended default
   - **Medium** (1.5 GB, ~5 GB RAM) — highest accuracy
   - Diarization models (~20 MB) are bundled automatically
3. Create a session (e.g., "Meeting Notes")
4. Upload an audio file (.mp3, .wav, .m4a, .ogg, .webm)
5. Wait for transcription to complete
6. View transcript and insights
7. Export to Markdown or PDF

## Architecture Notes

- **Frontend ↔ Rust**: All communication via Tauri IPC commands
  (see contracts/tauri-commands.md)
- **Audio playback**: Handled in Rust via rodio/symphonia for
  cross-platform format consistency
- **Transcription**: Local-only via sherpa-rs (sherpa-onnx) —
  Whisper ONNX for STT, pyannote + 3dspeaker for diarization
- **Storage**: SQLite with FTS5 for metadata + search; filesystem
  for audio files and downloaded models

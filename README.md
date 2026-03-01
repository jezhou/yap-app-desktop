<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" alt="Yap" width="96" />
</p>

<h1 align="center">Yap</h1>

<p align="center">
  <strong>Local voice conversation transcription.</strong><br/>
  Upload audio. Get transcripts. No cloud. No API keys. Fully offline.
</p>

<p align="center">
  <img alt="Platform" src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green" />
  <img alt="Version" src="https://img.shields.io/badge/version-0.1.0-orange" />
</p>

---

Yap is a cross-platform desktop app that transcribes and diarizes voice conversations entirely on your device. Powered by Whisper (via sherpa-onnx), it identifies speakers and produces searchable, exportable transcripts — all without ever leaving your machine.

## Features

- **Fully offline** — no network calls, no telemetry, no accounts
- **Speaker diarization** — automatically identifies and labels different speakers
- **Full-text search** — find anything across all your sessions and transcripts
- **PDF export** — export transcripts with speaker labels
- **Multi-format support** — MP3, WAV, OGG, AAC, M4A
- **Session management** — organize conversations into sessions with summaries
- **Lightweight** — <150 MB idle memory, <2s cold start

## Tech Stack

| Layer          | Technology                                           |
| -------------- | ---------------------------------------------------- |
| Framework      | [Tauri v2](https://v2.tauri.app/) (Rust + WebView)   |
| Frontend       | React 19, TypeScript 5, Vite 6, Tailwind CSS 4       |
| Transcription  | sherpa-onnx (Whisper ONNX) via sherpa-rs              |
| Diarization    | pyannote + 3dspeaker (via sherpa-rs)                  |
| Storage        | SQLite with FTS5 full-text search                     |
| Audio          | rodio / symphonia                                     |

## Getting Started

### Prerequisites

- **Rust** (latest stable via [rustup](https://rustup.rs/))
- **Node.js** 20+ and **pnpm**
- **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev libssl-dev`
- **macOS**: Xcode Command Line Tools
- **Windows**: Visual Studio Build Tools, WebView2

### Run

```bash
pnpm install        # install frontend deps
pnpm tauri dev      # launch in dev mode
```

### Build

```bash
pnpm tauri build    # production build
```

### Test

```bash
pnpm test                     # frontend (Vitest)
cd src-tauri && cargo test     # backend (Rust)
```

## Project Structure

```
src/                 # React frontend
├── components/      # UI components
├── pages/           # Route pages
├── services/        # Tauri IPC wrappers
└── types/           # Shared TypeScript types

src-tauri/           # Rust backend
├── src/
│   ├── commands/    # Tauri IPC handlers
│   ├── services/    # Business logic
│   ├── models/      # Data models
│   └── db/          # SQLite + migrations
```

## License

MIT

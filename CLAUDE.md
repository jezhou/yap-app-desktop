# yap-app-desktop Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-03-01

## Active Technologies

- Rust (latest stable) + TypeScript 5.x (001-voice-transcription)
- Tauri v2, React 19, Vite 6, Tailwind CSS 4 (001-voice-transcription)
- sherpa-rs for local STT + diarization (001-voice-transcription)
- SQLite via tauri-plugin-sql with FTS5 (001-voice-transcription)

## Project Structure

```text
src/                    # React frontend (TypeScript)
src-tauri/              # Rust backend (Tauri)
tests/                  # Frontend tests
specs/                  # Feature specifications
```

## Commands

```bash
pnpm tauri dev          # Run app in dev mode
pnpm test               # Frontend tests (Vitest)
cargo test              # Rust backend tests (from src-tauri/)
cargo clippy            # Rust linting (from src-tauri/)
pnpm tauri build        # Production build
```

## Code Style

- Rust: Follow standard Rust conventions (cargo fmt, cargo clippy)
- TypeScript/React: Follow standard React conventions
- Use conventional commits for git messages

## Recent Changes

- 001-voice-transcription: Tauri v2 + React + sherpa-rs (local-only STT + diarization) desktop app

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->

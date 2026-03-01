# Research: Voice Conversation Transcription

**Branch**: `001-voice-transcription` | **Date**: 2026-03-01

## Desktop Framework

**Decision**: Tauri v2 (latest stable, 2.10.x)

**Rationale**: Tauri meets all constitution constraints with wide
margins. Bundle size ~5-10 MB (vs Electron ~100+ MB). Idle memory
~20-40 MB (vs Electron ~200-300 MB, exceeding 150 MB ceiling).
Startup <500 ms. Uses OS-native webviews and Rust backend, aligning
with "lightweight by design" and "prefer native platform APIs."

**Alternatives considered**:
- **Electron**: Rejected. Idle memory (200-300 MB) violates the
  constitution's <150 MB constraint. Bundle size ~100 MB violates
  "keep as small as reasonably possible." Startup 1-2s leaves no
  safety margin.

**Stack details**:

| Component          | Choice                                      |
|--------------------|---------------------------------------------|
| Framework          | Tauri v2 (2.10.x)                           |
| Backend language   | Rust (latest stable)                        |
| Frontend           | React 19 + TypeScript 5.x                   |
| Build tool         | Vite 6.x                                    |
| UI styling         | Tailwind CSS 4.x (dark mode built-in)       |
| Package manager    | pnpm                                        |

**Known trade-off**: Tauri's webview on macOS (WebKit) does not
support .ogg audio playback. Mitigation: use Rust-side audio
playback via `rodio`/`symphonia` crate for cross-platform format
consistency.

## Local Transcription + Diarization

**Decision**: sherpa-onnx via `sherpa-rs` Rust crate (unified
framework for both STT and speaker diarization)

**Rationale**: sherpa-onnx provides speech-to-text, speaker
diarization, and VAD in a single framework with Rust bindings
(`sherpa-rs`). This eliminates the need for separate libraries
for transcription and diarization. Key advantages:
- Runs Whisper ONNX models (same weights = same accuracy as
  whisper.cpp, 2-5% WER)
- Built-in speaker diarization via pyannote segmentation +
  3dspeaker embedding models (~5-20 MB each)
- Fully offline, no Python dependency, no internet required
- Single `cargo add sherpa-rs` dependency
- Diarization example is ~48 lines of Rust
- Cross-platform: macOS, Windows, Linux
- Apache 2.0 license

**Alternatives considered**:
- **whisper-rs (whisper.cpp)**: Strong STT but no diarization
  support at all. Would require a separate diarization library,
  adding integration complexity.
- **faster-whisper**: Rejected. Requires bundling Python runtime
  (~200+ MB). Same accuracy but far worse desktop integration.
- **Vosk**: Rejected. 10-15% WER is unacceptable. No diarization.
- **whisper-rs + pyannote.audio**: Rejected. pyannote requires
  Python runtime (~200-300 MB bundle impact). sherpa-onnx uses
  the same pyannote segmentation model via ONNX without Python.

**Models for local transcription**:
- Whisper ONNX models (same sizes as whisper.cpp):
  - Default: `small` (466 MB, ~2 GB RAM)
  - Lightweight: `base` (142 MB, ~1 GB RAM)
  - High quality: `medium` (1.5 GB, ~5 GB RAM)

**Models for diarization** (bundled, small):
- Segmentation: `sherpa-onnx-pyannote-segmentation-3-0` (~5-10 MB)
- Speaker embedding: `3dspeaker_speech_eres2net_base` (~15 MB)

**Audio format handling**: Input audio converted to 16kHz mono
WAV via `symphonia` Rust crate before processing. Supports all
spec formats (.mp3, .wav, .m4a, .ogg, .webm).

## Local Data Storage

**Decision**: SQLite via `tauri-plugin-sql` with FTS5

**Rationale**: SQLite is the standard for local structured data.
FTS5 provides built-in full-text search satisfying FR-008. Single
file database, ACID transactions, zero config. Cross-platform
identical behavior. Minimal bundle impact (~600KB).

**Alternatives considered**:
- **IndexedDB**: Rejected. No built-in full-text search, no SQL,
  behavior varies across webview engines, harder to inspect/backup.
- **JSON files**: Rejected. No query capability, no transactions,
  search requires loading all files into memory.

**Fallback**: If sqlx FTS5 type-inference issues arise, switch to
`tauri-plugin-rusqlite2` (drop-in replacement using `rusqlite`).

## PDF Export

**Decision**: pdfmake (lazy-loaded on export action)

**Rationale**: Declarative JSON API maps 1:1 to transcript
document structure (title, summary, timestamped speaker segments).
Excellent quality. ~300KB gzipped but lazy-loaded, so zero impact
on startup. Cross-platform with no native dependencies.

**Alternatives considered**:
- **genpdf (Rust crate)**: Viable but less ergonomic API, more
  effort for equivalent visual quality. Good fallback if frontend
  bundle size becomes critical.
- **jsPDF**: Too low-level for structured documents.
- **window.print()**: Not viable — requires user interaction,
  no programmatic API in Tauri.
- **Puppeteer**: Rejected — spawns a browser process, antithetical
  to "lightweight by design."

## Audio Playback

**Decision**: Rust-side via `rodio`/`symphonia` crate

**Rationale**: Cross-platform format consistency. WebKit on macOS
lacks .ogg support; Rust-side playback handles all formats
uniformly. Better control over playback (seek, pause) than
browser <audio> element.

## Testing

| Layer      | Tool                               |
|------------|------------------------------------|
| Frontend   | Vitest + @tauri-apps/api/mocks     |
| Rust       | cargo test                         |
| E2E        | WebdriverIO via tauri-driver       |
| CI         | GitHub Actions matrix (macOS, Windows, Linux) |

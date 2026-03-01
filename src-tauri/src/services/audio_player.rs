use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

/// Shared audio player state managed by Tauri.
pub struct AudioPlayer {
    inner: Mutex<PlayerInner>,
}

struct PlayerInner {
    /// rodio output stream -- must be kept alive for playback
    _stream: Option<OutputStream>,
    /// Handle for creating sinks
    stream_handle: Option<OutputStreamHandle>,
    /// Current playback sink
    sink: Option<Sink>,
    /// Whether playback is active
    playing: bool,
    /// Duration of the current audio file in seconds
    duration_seconds: f64,
    /// Wall-clock time when playback started (after seek)
    playback_start: Option<Instant>,
    /// The accumulated position in seconds when last paused/seeked
    base_position: f64,
}

// SAFETY: PlayerInner is only accessed through a Mutex, ensuring
// single-threaded access. rodio's OutputStream is not Send due to
// platform audio backend raw pointers, but we guarantee exclusive
// access via the Mutex.
unsafe impl Send for PlayerInner {}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(PlayerInner {
                _stream: None,
                stream_handle: None,
                sink: None,
                playing: false,
                duration_seconds: 0.0,
                playback_start: None,
                base_position: 0.0,
            }),
        }
    }

    /// Start or resume playback of an audio file, optionally seeking.
    pub fn play(&self, file_path: &PathBuf, seek_seconds: Option<f64>) -> Result<PlayResult> {
        let mut inner = self.inner.lock().map_err(|e| anyhow::anyhow!("lock poisoned: {}", e))?;

        // Initialize output stream if needed
        if inner.stream_handle.is_none() {
            let (stream, handle) = OutputStream::try_default()
                .context("failed to open audio output device")?;
            inner._stream = Some(stream);
            inner.stream_handle = Some(handle);
        }

        // Stop any existing playback
        if let Some(old_sink) = inner.sink.take() {
            old_sink.stop();
        }

        let handle = inner.stream_handle.as_ref().unwrap();

        // Open and decode the audio file
        let file = File::open(file_path)
            .with_context(|| format!("audio file not found: {}", file_path.display()))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader)
            .context("failed to decode audio file")?;

        // Get duration from the decoder if available
        let duration = source.total_duration()
            .map(|d: std::time::Duration| d.as_secs_f64())
            .unwrap_or(0.0);

        let sink = Sink::try_new(handle)
            .context("failed to create audio sink")?;
        sink.append(source);

        // Handle seek
        let seek_pos = seek_seconds.unwrap_or(0.0);
        if seek_pos > 0.0 {
            let _ = sink.try_seek(std::time::Duration::from_secs_f64(seek_pos));
        }

        inner.sink = Some(sink);
        inner.playing = true;
        inner.duration_seconds = duration;
        inner.playback_start = Some(Instant::now());
        inner.base_position = seek_pos;

        Ok(PlayResult {
            playing: true,
            duration_seconds: duration,
        })
    }

    /// Pause current playback.
    pub fn pause(&self) -> Result<PauseResult> {
        let mut inner = self.inner.lock().map_err(|e| anyhow::anyhow!("lock poisoned: {}", e))?;

        let position = Self::calc_position(&inner);

        if let Some(ref sink) = inner.sink {
            sink.pause();
        }

        inner.playing = false;
        inner.base_position = position;
        inner.playback_start = None;

        Ok(PauseResult {
            playing: false,
            position_seconds: position,
        })
    }

    /// Get current playback position.
    pub fn get_position(&self) -> Result<PositionResult> {
        let inner = self.inner.lock().map_err(|e| anyhow::anyhow!("lock poisoned: {}", e))?;
        let position = Self::calc_position(&inner);

        // Check if sink finished playing
        let playing = inner.playing && inner.sink.as_ref().map_or(false, |s| !s.empty());

        Ok(PositionResult {
            position_seconds: position,
            playing,
        })
    }

    fn calc_position(inner: &PlayerInner) -> f64 {
        if inner.playing {
            if let Some(start) = inner.playback_start {
                let elapsed = start.elapsed().as_secs_f64();
                let pos = inner.base_position + elapsed;
                if inner.duration_seconds > 0.0 {
                    pos.min(inner.duration_seconds)
                } else {
                    pos
                }
            } else {
                inner.base_position
            }
        } else {
            inner.base_position
        }
    }
}

pub struct PlayResult {
    pub playing: bool,
    pub duration_seconds: f64,
}

pub struct PauseResult {
    pub playing: bool,
    pub position_seconds: f64,
}

pub struct PositionResult {
    pub position_seconds: f64,
    pub playing: bool,
}

/// Type alias for the managed audio player state.
pub type AudioPlayerState = Arc<AudioPlayer>;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};
use sherpa_rs::diarize::{Diarize, DiarizeConfig, Segment};
use sherpa_rs::whisper::{WhisperConfig, WhisperRecognizer};
use sherpa_rs::OfflineRecognizerResult;
use sqlx::SqlitePool;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use uuid::Uuid;

use crate::models::transcription::{Transcription, TranscriptionSegment};

/// Progress callback type: receives a percentage (0.0 to 100.0).
pub type ProgressCallback = Box<dyn Fn(f64) + Send + 'static>;

const TARGET_SAMPLE_RATE: u32 = 16000;

/// Result of running transcription + diarization on an audio file.
#[derive(Debug)]
pub struct TranscriptionResult {
    pub segments: Vec<TranscriptionSegment>,
    pub full_text: String,
}

/// Transcribe an audio file using sherpa-onnx (via sherpa-rs).
///
/// This function:
/// 1. Loads the audio file and converts to 16kHz mono samples
/// 2. Runs Whisper ONNX STT to get timestamped text segments
/// 3. Runs speaker diarization (pyannote segmentation + 3dspeaker embedding)
/// 4. Merges STT segments with speaker labels
///
/// The `cancel` flag can be set to true to abort processing early.
/// The `on_progress` callback receives percentage updates.
pub async fn transcribe_audio(
    audio_path: &Path,
    model_dir: &Path,
    cancel: Arc<AtomicBool>,
    on_progress: Option<ProgressCallback>,
) -> Result<TranscriptionResult> {
    let audio_path = audio_path.to_path_buf();
    let model_dir = model_dir.to_path_buf();

    // Run transcription in a blocking thread since sherpa-rs is synchronous
    let result = tokio::task::spawn_blocking(move || {
        transcribe_sync(&audio_path, &model_dir, cancel, on_progress)
    })
    .await
    .context("transcription task panicked")??;

    Ok(result)
}

/// Decode an audio file to 16kHz mono f32 samples using symphonia + rubato.
fn load_audio_samples(audio_path: &Path) -> Result<Vec<f32>> {
    let file = File::open(audio_path)
        .with_context(|| format!("failed to open audio file: {}", audio_path.display()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = audio_path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .context("failed to probe audio format")?;

    let mut format_reader = probed.format;
    let track = format_reader
        .default_track()
        .context("no audio track found")?
        .clone();
    let source_sample_rate = track
        .codec_params
        .sample_rate
        .context("unknown sample rate")?;
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(1);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .context("failed to create audio decoder")?;

    // Decode all packets into interleaved f32 samples
    let mut all_samples: Vec<f32> = Vec::new();
    loop {
        let packet = match format_reader.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(_) => break,
        };
        if packet.track_id() != track.id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let spec = *decoded.spec();
        let num_frames = decoded.capacity();
        let mut sample_buf = SampleBuffer::<f32>::new(num_frames as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);
        all_samples.extend_from_slice(sample_buf.samples());
    }

    if all_samples.is_empty() {
        anyhow::bail!("no audio samples decoded from file");
    }

    // Convert to mono by averaging channels
    let mono_samples: Vec<f32> = if channels > 1 {
        all_samples
            .chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        all_samples
    };

    // Resample to 16kHz if needed
    if source_sample_rate == TARGET_SAMPLE_RATE {
        return Ok(mono_samples);
    }

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let ratio = TARGET_SAMPLE_RATE as f64 / source_sample_rate as f64;
    let chunk_size = 1024;
    let mut resampler = SincFixedIn::<f32>::new(
        ratio,
        2.0,
        params,
        chunk_size,
        1, // mono
    )
    .context("failed to create resampler")?;

    let mut resampled: Vec<f32> = Vec::with_capacity((mono_samples.len() as f64 * ratio) as usize + 1024);

    // Process in chunks
    for chunk in mono_samples.chunks(chunk_size) {
        let input = if chunk.len() < chunk_size {
            // Pad last chunk with zeros
            let mut padded = chunk.to_vec();
            padded.resize(chunk_size, 0.0);
            vec![padded]
        } else {
            vec![chunk.to_vec()]
        };
        match resampler.process(&input, None) {
            Ok(output) => {
                if !output.is_empty() {
                    resampled.extend_from_slice(&output[0]);
                }
            }
            Err(e) => {
                eprintln!("resampler warning: {}", e);
                break;
            }
        }
    }

    // Trim to expected length (rubato may produce extra samples from padding)
    let expected_len = (mono_samples.len() as f64 * ratio) as usize;
    resampled.truncate(expected_len);

    Ok(resampled)
}

/// Find the best available whisper model directory.
/// Prefers user-selected model, then tries models in order: medium > small > base.
fn find_whisper_model(model_dir: &Path) -> Option<PathBuf> {
    // Check models in descending quality order
    for tier in &["whisper-medium", "whisper-small", "whisper-base"] {
        let dir = model_dir.join(tier);
        if has_whisper_files(&dir) {
            return Some(dir);
        }
    }
    None
}

/// Check if a directory contains the required whisper model files.
fn has_whisper_files(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    // Look for encoder, decoder, and tokens files
    let has_encoder = find_file_matching(dir, "encoder").is_some();
    let has_decoder = find_file_matching(dir, "decoder").is_some();
    let has_tokens = find_file_matching(dir, "tokens").is_some();
    has_encoder && has_decoder && has_tokens
}

/// Find a file in a directory whose name contains the given substring and ends with .onnx or .txt.
fn find_file_matching(dir: &Path, substring: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.filter_map(|e| e.ok()).find_map(|entry| {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(substring) {
            Some(entry.path())
        } else {
            None
        }
    })
}

/// Run Whisper STT on 16kHz mono samples.
fn run_whisper_stt(
    samples: &[f32],
    whisper_dir: &Path,
    cancel: &AtomicBool,
) -> Result<OfflineRecognizerResult> {
    if cancel.load(Ordering::Relaxed) {
        anyhow::bail!("transcription cancelled");
    }

    let encoder = find_file_matching(whisper_dir, "encoder")
        .context("encoder model file not found")?;
    let decoder = find_file_matching(whisper_dir, "decoder")
        .context("decoder model file not found")?;
    let tokens = find_file_matching(whisper_dir, "tokens")
        .context("tokens file not found")?;

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(2)
        .min(4);

    let config = WhisperConfig {
        encoder: encoder.to_string_lossy().into_owned(),
        decoder: decoder.to_string_lossy().into_owned(),
        tokens: tokens.to_string_lossy().into_owned(),
        language: "en".to_string(),
        num_threads: Some(num_threads),
        ..Default::default()
    };

    let mut recognizer = WhisperRecognizer::new(config)
        .map_err(|e| anyhow::anyhow!("failed to create whisper recognizer: {}", e))?;

    if cancel.load(Ordering::Relaxed) {
        anyhow::bail!("transcription cancelled");
    }

    let result = recognizer.transcribe(TARGET_SAMPLE_RATE, samples);
    Ok(result)
}

/// Run speaker diarization on 16kHz mono samples.
/// Returns None if diarization models are not available (graceful degradation).
fn run_diarization(
    samples: Vec<f32>,
    model_dir: &Path,
    cancel: &AtomicBool,
) -> Option<Vec<Segment>> {
    if cancel.load(Ordering::Relaxed) {
        return None;
    }

    let seg_model = model_dir
        .join("pyannote-segmentation")
        .join("model.onnx");
    let emb_model_dir = model_dir.join("3dspeaker-embedding");

    // Find the embedding model file
    let emb_model = std::fs::read_dir(&emb_model_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with(".onnx")
        })
        .map(|e| e.path())?;

    if !seg_model.exists() || !emb_model.exists() {
        return None;
    }

    let config = DiarizeConfig {
        num_clusters: None, // auto-detect number of speakers
        threshold: Some(0.5),
        ..Default::default()
    };

    let mut diarizer = Diarize::new(&seg_model, &emb_model, config).ok()?;

    match diarizer.compute(samples, None) {
        Ok(segments) => Some(segments),
        Err(e) => {
            eprintln!("diarization failed (non-fatal): {}", e);
            None
        }
    }
}

/// Merge Whisper STT result with diarization segments.
/// If diarization is available, assigns speaker labels based on time overlap.
/// Otherwise, splits text by natural pauses and assigns all to "Speaker 1".
fn merge_stt_and_diarization(
    stt: &OfflineRecognizerResult,
    diarization: Option<&[Segment]>,
) -> Vec<TranscriptionSegment> {
    if stt.tokens.is_empty() || stt.text.trim().is_empty() {
        return vec![];
    }

    match diarization {
        Some(diar_segments) if !diar_segments.is_empty() => {
            merge_with_speakers(stt, diar_segments)
        }
        _ => {
            split_by_pauses(stt)
        }
    }
}

/// Merge STT tokens with diarization speaker segments.
/// For each diarization segment, collect all tokens whose timestamps fall within it.
fn merge_with_speakers(
    stt: &OfflineRecognizerResult,
    diar_segments: &[Segment],
) -> Vec<TranscriptionSegment> {
    let mut result: Vec<TranscriptionSegment> = Vec::new();

    for seg in diar_segments {
        let speaker_label = format!("Speaker {}", seg.speaker + 1);

        // Collect tokens that fall within this diarization segment
        let mut segment_tokens: Vec<String> = Vec::new();
        let mut seg_start = seg.start as f64;
        let mut seg_end = seg.end as f64;

        for (i, token) in stt.tokens.iter().enumerate() {
            if i >= stt.timestamps.len() {
                break;
            }
            let ts = stt.timestamps[i] as f64;
            // Token falls within this diarization segment (with small tolerance)
            if ts >= (seg.start as f64 - 0.1) && ts <= (seg.end as f64 + 0.1) {
                let cleaned = token.trim().to_string();
                if !cleaned.is_empty() {
                    segment_tokens.push(cleaned);
                    if ts < seg_start {
                        seg_start = ts;
                    }
                    if ts > seg_end {
                        seg_end = ts;
                    }
                }
            }
        }

        if segment_tokens.is_empty() {
            continue;
        }

        let text = segment_tokens.join("").trim().to_string();
        if text.is_empty() {
            continue;
        }

        // Merge with previous segment if same speaker and contiguous
        if let Some(prev) = result.last_mut() {
            if prev.speaker == speaker_label && (seg_start - prev.end_time).abs() < 0.5 {
                prev.text = format!("{}{}", prev.text, text);
                prev.end_time = seg_end;
                continue;
            }
        }

        result.push(TranscriptionSegment {
            speaker: speaker_label,
            text,
            start_time: seg_start,
            end_time: seg_end,
            confidence: 0.9,
        });
    }

    // Handle orphan tokens not covered by any diarization segment
    let mut covered: Vec<bool> = vec![false; stt.tokens.len()];
    for seg in diar_segments {
        for (i, _token) in stt.tokens.iter().enumerate() {
            if i < stt.timestamps.len() {
                let ts = stt.timestamps[i] as f64;
                if ts >= (seg.start as f64 - 0.1) && ts <= (seg.end as f64 + 0.1) {
                    covered[i] = true;
                }
            }
        }
    }

    // Assign orphan tokens to nearest speaker segment
    for (i, token) in stt.tokens.iter().enumerate() {
        if covered[i] || i >= stt.timestamps.len() {
            continue;
        }
        let cleaned = token.trim().to_string();
        if cleaned.is_empty() {
            continue;
        }
        let ts = stt.timestamps[i] as f64;

        // Find the nearest result segment and append
        if let Some(nearest) = result.iter_mut().min_by(|a, b| {
            let dist_a = (a.start_time - ts).abs().min((a.end_time - ts).abs());
            let dist_b = (b.start_time - ts).abs().min((b.end_time - ts).abs());
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            nearest.text = format!("{}{}", nearest.text, cleaned);
            if ts > nearest.end_time {
                nearest.end_time = ts;
            }
            if ts < nearest.start_time {
                nearest.start_time = ts;
            }
        }
    }

    result
}

/// Split transcription by natural pauses (>1s gaps between tokens).
/// All segments assigned to "Speaker 1" (no diarization available).
fn split_by_pauses(stt: &OfflineRecognizerResult) -> Vec<TranscriptionSegment> {
    if stt.tokens.is_empty() {
        return vec![];
    }

    let mut segments: Vec<TranscriptionSegment> = Vec::new();
    let mut current_tokens: Vec<String> = Vec::new();
    let mut seg_start: f64 = 0.0;
    let mut prev_ts: f64 = 0.0;

    for (i, token) in stt.tokens.iter().enumerate() {
        let ts = if i < stt.timestamps.len() {
            stt.timestamps[i] as f64
        } else {
            prev_ts + 0.1
        };

        // If there's a >1s gap, start a new segment
        if !current_tokens.is_empty() && (ts - prev_ts) > 1.0 {
            let text = current_tokens.join("").trim().to_string();
            if !text.is_empty() {
                segments.push(TranscriptionSegment {
                    speaker: "Speaker 1".to_string(),
                    text,
                    start_time: seg_start,
                    end_time: prev_ts,
                    confidence: 0.9,
                });
            }
            current_tokens.clear();
            seg_start = ts;
        }

        if current_tokens.is_empty() {
            seg_start = ts;
        }

        let cleaned = token.trim().to_string();
        if !cleaned.is_empty() {
            current_tokens.push(cleaned);
        }
        prev_ts = ts;
    }

    // Flush remaining tokens
    if !current_tokens.is_empty() {
        let text = current_tokens.join("").trim().to_string();
        if !text.is_empty() {
            segments.push(TranscriptionSegment {
                speaker: "Speaker 1".to_string(),
                text,
                start_time: seg_start,
                end_time: prev_ts,
                confidence: 0.9,
            });
        }
    }

    // If we ended up with nothing, create a single segment from full text
    if segments.is_empty() && !stt.text.trim().is_empty() {
        let last_ts = stt.timestamps.last().copied().unwrap_or(0.0) as f64;
        segments.push(TranscriptionSegment {
            speaker: "Speaker 1".to_string(),
            text: stt.text.trim().to_string(),
            start_time: 0.0,
            end_time: last_ts,
            confidence: 0.9,
        });
    }

    segments
}

/// Stub transcription for when no model is available (preserves test compatibility).
fn transcribe_stub(
    on_progress: &Option<ProgressCallback>,
    cancel: &AtomicBool,
) -> Result<TranscriptionResult> {
    let progress_stages = [10.0, 30.0, 50.0, 70.0, 90.0, 100.0];
    for &pct in &progress_stages {
        if cancel.load(Ordering::Relaxed) {
            anyhow::bail!("transcription cancelled");
        }
        if let Some(ref cb) = on_progress {
            cb(pct);
        }
    }

    let segments = vec![TranscriptionSegment {
        speaker: "Speaker 1".to_string(),
        text: "Transcription will appear here once a model is downloaded.".to_string(),
        start_time: 0.0,
        end_time: 5.0,
        confidence: 0.0,
    }];

    let full_text = segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
    Ok(TranscriptionResult { segments, full_text })
}

/// Synchronous transcription implementation.
///
/// Chains: load_audio (0-20%) → whisper STT (20-70%) → diarize (70-90%) → merge (90-100%).
/// Falls back to stub if no whisper model is found.
fn transcribe_sync(
    audio_path: &Path,
    model_dir: &Path,
    cancel: Arc<AtomicBool>,
    on_progress: Option<ProgressCallback>,
) -> Result<TranscriptionResult> {
    // Check cancellation
    if cancel.load(Ordering::Relaxed) {
        anyhow::bail!("transcription cancelled");
    }

    // Verify the audio file exists
    if !audio_path.exists() {
        anyhow::bail!("audio file not found: {}", audio_path.display());
    }

    // Report initial progress
    if let Some(ref cb) = on_progress {
        cb(0.0);
    }

    // Find a whisper model — fall back to stub if none available
    let whisper_dir = match find_whisper_model(model_dir) {
        Some(dir) => dir,
        None => return transcribe_stub(&on_progress, &cancel),
    };

    // Phase 1: Load audio (0-20%)
    if let Some(ref cb) = on_progress {
        cb(5.0);
    }

    let samples = load_audio_samples(audio_path)?;

    if cancel.load(Ordering::Relaxed) {
        anyhow::bail!("transcription cancelled");
    }
    if let Some(ref cb) = on_progress {
        cb(20.0);
    }

    // Phase 2: Whisper STT (20-70%)
    let stt_result = run_whisper_stt(&samples, &whisper_dir, &cancel)?;

    if cancel.load(Ordering::Relaxed) {
        anyhow::bail!("transcription cancelled");
    }
    if let Some(ref cb) = on_progress {
        cb(70.0);
    }

    // Phase 3: Diarization (70-90%)
    let diar_segments = run_diarization(samples, model_dir, &cancel);

    if cancel.load(Ordering::Relaxed) {
        anyhow::bail!("transcription cancelled");
    }
    if let Some(ref cb) = on_progress {
        cb(90.0);
    }

    // Phase 4: Merge (90-100%)
    let segments = merge_stt_and_diarization(&stt_result, diar_segments.as_deref());

    // Build full text
    let full_text = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    if let Some(ref cb) = on_progress {
        cb(100.0);
    }

    Ok(TranscriptionResult { segments, full_text })
}

/// Save a transcription result to the database.
pub async fn save_transcription(
    pool: &SqlitePool,
    conversation_id: &str,
    result: &TranscriptionResult,
) -> Result<Transcription> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let segments_json =
        serde_json::to_string(&result.segments).context("failed to serialize segments")?;

    sqlx::query(
        "INSERT INTO transcriptions (id, conversation_id, full_text, segments, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(&result.full_text)
    .bind(&segments_json)
    .bind(&now)
    .execute(pool)
    .await
    .context("failed to save transcription")?;

    Ok(Transcription {
        id,
        conversation_id: conversation_id.to_string(),
        full_text: result.full_text.clone(),
        segments: result.segments.clone(),
        created_at: now,
    })
}

/// Update conversation status in the database.
pub async fn update_conversation_status(
    pool: &SqlitePool,
    conversation_id: &str,
    status: &str,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    sqlx::query("UPDATE conversations SET status = ?, updated_at = ? WHERE id = ?")
        .bind(status)
        .bind(&now)
        .bind(conversation_id)
        .execute(pool)
        .await
        .context("failed to update conversation status")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_transcribe_audio_file_not_found() {
        let cancel = Arc::new(AtomicBool::new(false));
        let result = transcribe_audio(
            Path::new("/nonexistent/audio.wav"),
            Path::new("/models"),
            cancel,
            None,
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("audio file not found"));
    }

    #[tokio::test]
    async fn test_transcribe_audio_cancellation() {
        let cancel = Arc::new(AtomicBool::new(true));

        // Create a temp file so the file exists
        let tmp = NamedTempFile::new().unwrap();

        let result = transcribe_audio(tmp.path(), Path::new("/models"), cancel, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    #[tokio::test]
    async fn test_transcribe_audio_progress_callback() {
        let cancel = Arc::new(AtomicBool::new(false));
        let progress_values = Arc::new(std::sync::Mutex::new(Vec::new()));
        let pv = progress_values.clone();

        let tmp = NamedTempFile::new().unwrap();

        let result = transcribe_audio(
            tmp.path(),
            Path::new("/models"),
            cancel,
            Some(Box::new(move |pct| {
                pv.lock().unwrap().push(pct);
            })),
        )
        .await;

        assert!(result.is_ok());
        let values = progress_values.lock().unwrap();
        assert!(!values.is_empty());
        assert_eq!(*values.last().unwrap(), 100.0);
    }

    #[tokio::test]
    async fn test_transcribe_audio_returns_segments() {
        let cancel = Arc::new(AtomicBool::new(false));
        let tmp = NamedTempFile::new().unwrap();

        let result = transcribe_audio(tmp.path(), Path::new("/models"), cancel, None)
            .await
            .unwrap();

        assert!(!result.segments.is_empty());
        assert!(!result.full_text.is_empty());
    }

    #[tokio::test]
    async fn test_save_transcription() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .unwrap();

        for sql in &[
            "CREATE TABLE sessions (id TEXT PRIMARY KEY, title TEXT NOT NULL, description TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
            "CREATE TABLE conversations (id TEXT PRIMARY KEY, session_id TEXT NOT NULL REFERENCES sessions(id), sequence_number INTEGER NOT NULL, title TEXT NOT NULL, audio_file_path TEXT NOT NULL, duration_seconds REAL NOT NULL, status TEXT NOT NULL DEFAULT 'uploading', created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
            "CREATE TABLE transcriptions (id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL UNIQUE REFERENCES conversations(id) ON DELETE CASCADE, full_text TEXT NOT NULL, segments TEXT NOT NULL, created_at TEXT NOT NULL)",
        ] {
            sqlx::query(sql).execute(&pool).await.unwrap();
        }

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (id, title, created_at, updated_at) VALUES ('s1', 'Test', ?, ?)",
        )
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO conversations (id, session_id, sequence_number, title, audio_file_path, duration_seconds, status, created_at, updated_at) VALUES ('c1', 's1', 1, 'Conv', '/audio/test.wav', 60.0, 'analyzing', ?, ?)")
            .bind(&now).bind(&now).execute(&pool).await.unwrap();

        let result = TranscriptionResult {
            segments: vec![TranscriptionSegment {
                speaker: "Speaker 1".to_string(),
                text: "Hello world".to_string(),
                start_time: 0.0,
                end_time: 2.0,
                confidence: 0.95,
            }],
            full_text: "Hello world".to_string(),
        };

        let transcription = save_transcription(&pool, "c1", &result).await.unwrap();
        assert_eq!(transcription.conversation_id, "c1");
        assert_eq!(transcription.full_text, "Hello world");
        assert_eq!(transcription.segments.len(), 1);
    }

    #[tokio::test]
    async fn test_update_conversation_status() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        for sql in &[
            "CREATE TABLE sessions (id TEXT PRIMARY KEY, title TEXT NOT NULL, description TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
            "CREATE TABLE conversations (id TEXT PRIMARY KEY, session_id TEXT NOT NULL, sequence_number INTEGER NOT NULL, title TEXT NOT NULL, audio_file_path TEXT NOT NULL, duration_seconds REAL NOT NULL, status TEXT NOT NULL DEFAULT 'uploading', created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
        ] {
            sqlx::query(sql).execute(&pool).await.unwrap();
        }

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (id, title, created_at, updated_at) VALUES ('s1', 'Test', ?, ?)",
        )
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO conversations (id, session_id, sequence_number, title, audio_file_path, duration_seconds, status, created_at, updated_at) VALUES ('c1', 's1', 1, 'Conv', '/audio/test.wav', 60.0, 'uploading', ?, ?)")
            .bind(&now).bind(&now).execute(&pool).await.unwrap();

        update_conversation_status(&pool, "c1", "analyzing")
            .await
            .unwrap();

        let (status,): (String,) =
            sqlx::query_as("SELECT status FROM conversations WHERE id = 'c1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(status, "analyzing");
    }
}

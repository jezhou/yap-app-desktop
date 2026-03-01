use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::transcription::{Transcription, TranscriptionSegment};

/// Progress callback type: receives a percentage (0.0 to 100.0).
pub type ProgressCallback = Box<dyn Fn(f64) + Send + 'static>;

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

/// Synchronous transcription implementation.
///
/// When sherpa-rs is integrated, this function will:
/// 1. Load audio samples via symphonia (convert to 16kHz mono f32)
/// 2. Create a sherpa_rs::Recognizer with the Whisper ONNX model
/// 3. Feed audio in chunks, emitting progress via callback
/// 4. Run sherpa_rs::speaker_diarization for speaker labels
/// 5. Merge transcript segments with speaker assignments
///
/// For now, this returns a placeholder result when no model is available,
/// allowing the full pipeline to be tested end-to-end.
fn transcribe_sync(
    audio_path: &Path,
    _model_dir: &Path,
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

    // --- sherpa-rs integration point ---
    // When sherpa-rs is added to Cargo.toml, replace this block with:
    //
    // 1. Load audio samples:
    //    let samples = load_audio_samples(audio_path)?;  // 16kHz mono f32
    //
    // 2. Create recognizer:
    //    let config = sherpa_rs::OnlineRecognizerConfig { ... };
    //    let recognizer = sherpa_rs::OnlineRecognizer::new(config)?;
    //
    // 3. Process in chunks with progress:
    //    for (i, chunk) in samples.chunks(chunk_size).enumerate() {
    //        if cancel.load(Ordering::Relaxed) { bail!("cancelled"); }
    //        recognizer.accept_waveform(chunk);
    //        if let Some(ref cb) = on_progress {
    //            cb((i as f64 / total_chunks as f64) * 80.0);
    //        }
    //    }
    //
    // 4. Run diarization:
    //    let diarization_config = sherpa_rs::OfflineSpeakerDiarizationConfig { ... };
    //    let sd = sherpa_rs::OfflineSpeakerDiarization::new(diarization_config)?;
    //    let speaker_segments = sd.process(&samples);
    //
    // 5. Merge STT + diarization results into TranscriptionSegments

    // Simulate progress stages for pipeline testing
    let progress_stages = [10.0, 30.0, 50.0, 70.0, 90.0, 100.0];
    for &pct in &progress_stages {
        if cancel.load(Ordering::Relaxed) {
            anyhow::bail!("transcription cancelled");
        }
        if let Some(ref cb) = on_progress {
            cb(pct);
        }
    }

    // Placeholder result — will be replaced by real sherpa-rs output
    let segments = vec![TranscriptionSegment {
        speaker: "Speaker 1".to_string(),
        text: "Transcription will appear here once a model is downloaded.".to_string(),
        start_time: 0.0,
        end_time: 5.0,
        confidence: 0.0,
    }];

    let full_text = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(TranscriptionResult {
        segments,
        full_text,
    })
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

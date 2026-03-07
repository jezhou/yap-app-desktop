use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::transcription::{Transcription, TranscriptionSegment};

/// Progress callback type: receives a percentage (0.0 to 100.0).
pub type ProgressCallback = Box<dyn Fn(f64) + Send + 'static>;

const DEEPGRAM_API_URL: &str = "https://api.deepgram.com/v1/listen";

/// Result of running transcription on an audio file.
#[derive(Debug)]
pub struct TranscriptionResult {
    pub segments: Vec<TranscriptionSegment>,
    pub full_text: String,
}

// --- Deepgram response types ---

#[derive(Debug, Deserialize)]
struct DeepgramResponse {
    results: DeepgramResults,
}

#[derive(Debug, Deserialize)]
struct DeepgramResults {
    utterances: Option<Vec<DeepgramUtterance>>,
    channels: Vec<DeepgramChannel>,
}

#[derive(Debug, Deserialize)]
struct DeepgramUtterance {
    speaker: i32,
    transcript: String,
    start: f64,
    end: f64,
    confidence: f64,
}

#[derive(Debug, Deserialize)]
struct DeepgramChannel {
    alternatives: Vec<DeepgramAlternative>,
}

#[derive(Debug, Deserialize)]
struct DeepgramAlternative {
    transcript: String,
    words: Option<Vec<DeepgramWord>>,
}

#[derive(Debug, Deserialize)]
struct DeepgramWord {
    word: String,
    start: f64,
    end: f64,
    #[allow(dead_code)]
    confidence: f64,
    speaker: Option<i32>,
}

/// Detect MIME type from file extension.
fn mime_type_for_audio(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("m4a") | Some("mp4") | Some("aac") => "audio/mp4",
        Some("ogg") | Some("oga") => "audio/ogg",
        Some("webm") => "audio/webm",
        Some("flac") => "audio/flac",
        _ => "audio/wav",
    }
}

/// Map Deepgram utterances to our TranscriptionSegment format.
fn map_utterances(utterances: &[DeepgramUtterance]) -> Vec<TranscriptionSegment> {
    utterances
        .iter()
        .filter(|u| !u.transcript.trim().is_empty())
        .map(|u| TranscriptionSegment {
            speaker: format!("Speaker {}", u.speaker + 1),
            text: u.transcript.trim().to_string(),
            start_time: u.start,
            end_time: u.end,
            confidence: u.confidence,
        })
        .collect()
}

/// Fallback: build segments from channel words when utterances aren't available.
fn map_words_to_segments(words: &[DeepgramWord]) -> Vec<TranscriptionSegment> {
    if words.is_empty() {
        return vec![];
    }

    let mut segments: Vec<TranscriptionSegment> = Vec::new();
    let mut current_speaker = words[0].speaker.unwrap_or(0);
    let mut current_words: Vec<&str> = Vec::new();
    let mut seg_start = words[0].start;
    let mut seg_end = words[0].end;

    for word in words {
        let speaker = word.speaker.unwrap_or(0);
        if speaker != current_speaker && !current_words.is_empty() {
            segments.push(TranscriptionSegment {
                speaker: format!("Speaker {}", current_speaker + 1),
                text: current_words.join(" "),
                start_time: seg_start,
                end_time: seg_end,
                confidence: 0.9,
            });
            current_words.clear();
            seg_start = word.start;
            current_speaker = speaker;
        }
        current_words.push(&word.word);
        seg_end = word.end;
    }

    if !current_words.is_empty() {
        segments.push(TranscriptionSegment {
            speaker: format!("Speaker {}", current_speaker + 1),
            text: current_words.join(" "),
            start_time: seg_start,
            end_time: seg_end,
            confidence: 0.9,
        });
    }

    segments
}

/// Transcribe an audio file using the Deepgram API.
///
/// Sends the audio file to Deepgram's nova-2 model with diarization enabled.
/// The `cancel` flag can be set to true to abort processing.
/// The `on_progress` callback receives percentage updates.
pub async fn transcribe_audio(
    audio_path: &Path,
    api_key: &str,
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

    eprintln!("[transcription] audio file: {}", audio_path.display());

    // Progress: starting
    if let Some(ref cb) = on_progress {
        cb(0.0);
    }

    // Phase 1: Read the audio file (0-10%)
    eprintln!("[transcription] reading audio file...");
    if let Some(ref cb) = on_progress {
        cb(5.0);
    }

    let audio_bytes = tokio::fs::read(audio_path)
        .await
        .with_context(|| format!("failed to read audio file: {}", audio_path.display()))?;

    let content_type = mime_type_for_audio(audio_path);
    eprintln!(
        "[transcription] file size: {} bytes, content-type: {}",
        audio_bytes.len(),
        content_type
    );

    if cancel.load(Ordering::Relaxed) {
        anyhow::bail!("transcription cancelled");
    }

    if let Some(ref cb) = on_progress {
        cb(10.0);
    }

    // Phase 2: Send to Deepgram API (10-80%)
    eprintln!("[transcription] sending to Deepgram API...");
    if let Some(ref cb) = on_progress {
        cb(20.0);
    }

    let client = reqwest::Client::new();
    let response = client
        .post(DEEPGRAM_API_URL)
        .query(&[
            ("model", "nova-2"),
            ("smart_format", "true"),
            ("diarize", "true"),
            ("utterances", "true"),
            ("language", "en"),
        ])
        .header("Authorization", format!("Token {}", api_key))
        .header("Content-Type", content_type)
        .body(audio_bytes)
        .send()
        .await
        .context("failed to send request to Deepgram")?;

    if cancel.load(Ordering::Relaxed) {
        anyhow::bail!("transcription cancelled");
    }

    if let Some(ref cb) = on_progress {
        cb(70.0);
    }

    // Check for API errors
    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            anyhow::bail!("InvalidApiKey: Deepgram API key is invalid or expired. Please check your API key in Settings.");
        }
        anyhow::bail!(
            "Deepgram API error (HTTP {}): {}",
            status.as_u16(),
            error_body
        );
    }

    // Phase 3: Parse response (80-90%)
    eprintln!("[transcription] parsing Deepgram response...");
    if let Some(ref cb) = on_progress {
        cb(80.0);
    }

    let deepgram_response: DeepgramResponse = response
        .json::<DeepgramResponse>()
        .await
        .context("failed to parse Deepgram response")?;

    if cancel.load(Ordering::Relaxed) {
        anyhow::bail!("transcription cancelled");
    }

    // Phase 4: Map to our segment format (90-100%)
    if let Some(ref cb) = on_progress {
        cb(90.0);
    }

    let segments = if let Some(ref utterances) = deepgram_response.results.utterances {
        if !utterances.is_empty() {
            eprintln!(
                "[transcription] mapping {} utterances to segments",
                utterances.len()
            );
            map_utterances(utterances)
        } else {
            map_from_channels(&deepgram_response.results.channels)
        }
    } else {
        map_from_channels(&deepgram_response.results.channels)
    };

    let full_text = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    eprintln!(
        "[transcription] done: {} segments, {} chars of text",
        segments.len(),
        full_text.len()
    );

    if let Some(ref cb) = on_progress {
        cb(100.0);
    }

    Ok(TranscriptionResult { segments, full_text })
}

/// Fallback: extract segments from channel data when utterances aren't available.
fn map_from_channels(channels: &[DeepgramChannel]) -> Vec<TranscriptionSegment> {
    if let Some(channel) = channels.first() {
        if let Some(alt) = channel.alternatives.first() {
            // Try words with speaker info first
            if let Some(ref words) = alt.words {
                if !words.is_empty() {
                    return map_words_to_segments(words);
                }
            }
            // Last resort: single segment from full transcript
            if !alt.transcript.trim().is_empty() {
                return vec![TranscriptionSegment {
                    speaker: "Speaker 1".to_string(),
                    text: alt.transcript.trim().to_string(),
                    start_time: 0.0,
                    end_time: 0.0,
                    confidence: 0.9,
                }];
            }
        }
    }
    vec![]
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
    use tempfile::NamedTempFile;

    #[test]
    fn test_mime_type_detection() {
        assert_eq!(mime_type_for_audio(Path::new("test.mp3")), "audio/mpeg");
        assert_eq!(mime_type_for_audio(Path::new("test.wav")), "audio/wav");
        assert_eq!(mime_type_for_audio(Path::new("test.m4a")), "audio/mp4");
        assert_eq!(mime_type_for_audio(Path::new("test.ogg")), "audio/ogg");
        assert_eq!(mime_type_for_audio(Path::new("test.webm")), "audio/webm");
        assert_eq!(mime_type_for_audio(Path::new("test.flac")), "audio/flac");
        assert_eq!(mime_type_for_audio(Path::new("test.xyz")), "audio/wav");
    }

    #[test]
    fn test_map_utterances() {
        let utterances = vec![
            DeepgramUtterance {
                speaker: 0,
                transcript: "Hello there.".to_string(),
                start: 0.0,
                end: 1.5,
                confidence: 0.98,
            },
            DeepgramUtterance {
                speaker: 1,
                transcript: "Hi, how are you?".to_string(),
                start: 1.8,
                end: 3.2,
                confidence: 0.95,
            },
        ];

        let segments = map_utterances(&utterances);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].speaker, "Speaker 1");
        assert_eq!(segments[0].text, "Hello there.");
        assert_eq!(segments[0].start_time, 0.0);
        assert_eq!(segments[0].end_time, 1.5);
        assert_eq!(segments[1].speaker, "Speaker 2");
        assert_eq!(segments[1].text, "Hi, how are you?");
    }

    #[test]
    fn test_map_utterances_empty() {
        let segments = map_utterances(&[]);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_map_utterances_skips_empty_transcripts() {
        let utterances = vec![
            DeepgramUtterance {
                speaker: 0,
                transcript: "Hello".to_string(),
                start: 0.0,
                end: 1.0,
                confidence: 0.9,
            },
            DeepgramUtterance {
                speaker: 0,
                transcript: "  ".to_string(),
                start: 1.0,
                end: 2.0,
                confidence: 0.5,
            },
        ];

        let segments = map_utterances(&utterances);
        assert_eq!(segments.len(), 1);
    }

    #[test]
    fn test_map_words_to_segments() {
        let words = vec![
            DeepgramWord {
                word: "Hello".to_string(),
                start: 0.0,
                end: 0.5,
                confidence: 0.98,
                speaker: Some(0),
            },
            DeepgramWord {
                word: "there".to_string(),
                start: 0.5,
                end: 1.0,
                confidence: 0.97,
                speaker: Some(0),
            },
            DeepgramWord {
                word: "Hi".to_string(),
                start: 1.5,
                end: 2.0,
                confidence: 0.95,
                speaker: Some(1),
            },
        ];

        let segments = map_words_to_segments(&words);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].speaker, "Speaker 1");
        assert_eq!(segments[0].text, "Hello there");
        assert_eq!(segments[1].speaker, "Speaker 2");
        assert_eq!(segments[1].text, "Hi");
    }

    #[tokio::test]
    async fn test_transcribe_audio_file_not_found() {
        let cancel = Arc::new(AtomicBool::new(false));
        let result = transcribe_audio(
            Path::new("/nonexistent/audio.wav"),
            "fake-key",
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
        let tmp = NamedTempFile::new().unwrap();

        let result = transcribe_audio(tmp.path(), "fake-key", cancel, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
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

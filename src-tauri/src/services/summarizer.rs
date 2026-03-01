use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::summary::Summary;
use crate::models::transcription::TranscriptionSegment;

/// Generate a summary from transcription segments.
///
/// Produces a 1-3 sentence summary and bullet-point key insights.
/// Since all processing is local and we don't have a local LLM for
/// summarization, this uses extractive summarization — selecting
/// the most representative sentences from the transcript.
pub fn generate_summary(segments: &[TranscriptionSegment]) -> String {
    if segments.is_empty() {
        return "No speech detected in this recording.".to_string();
    }

    let full_text: String = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // Split into sentences
    let sentences: Vec<&str> = full_text
        .split(|c: char| c == '.' || c == '!' || c == '?')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if sentences.is_empty() {
        return format!(
            "Recording contains speech from {} speaker(s).",
            count_speakers(segments)
        );
    }

    // Build summary parts
    let mut parts = Vec::new();

    // Part 1: Overview sentence
    let speaker_count = count_speakers(segments);
    let duration = segments.last().map(|s| s.end_time).unwrap_or(0.0);
    let duration_str = format_duration(duration);

    if speaker_count > 1 {
        parts.push(format!(
            "Conversation between {} speakers spanning {}.",
            speaker_count, duration_str
        ));
    } else {
        parts.push(format!("Recording spanning {}.", duration_str));
    }

    // Part 2: Key points as bullet list
    // Extract up to 5 representative sentences, favoring longer ones
    // that are more likely to contain meaningful content
    let mut scored_sentences: Vec<(usize, &str)> =
        sentences.iter().enumerate().map(|(i, &s)| (i, s)).collect();

    // Score by word count (longer sentences tend to be more informative)
    scored_sentences.sort_by(|a, b| {
        let a_words = a.1.split_whitespace().count();
        let b_words = b.1.split_whitespace().count();
        b_words.cmp(&a_words)
    });

    let key_points: Vec<&str> = scored_sentences.iter().take(5).map(|(_, s)| *s).collect();

    if !key_points.is_empty() {
        parts.push("\nKey points:".to_string());
        for point in &key_points {
            // Truncate long points
            let truncated = if point.len() > 150 {
                format!("{}...", &point[..150])
            } else {
                point.to_string()
            };
            parts.push(format!("- {}", truncated));
        }
    }

    parts.join("\n")
}

/// Count the number of unique speakers in the segments.
fn count_speakers(segments: &[TranscriptionSegment]) -> usize {
    let mut speakers: Vec<&str> = segments.iter().map(|s| s.speaker.as_str()).collect();
    speakers.sort_unstable();
    speakers.dedup();
    speakers.len()
}

/// Format a duration in seconds to a human-readable string.
fn format_duration(seconds: f64) -> String {
    let total = seconds as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Save a summary to the database.
pub async fn save_summary(
    pool: &SqlitePool,
    conversation_id: &str,
    content: &str,
) -> Result<Summary> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO summaries (id, conversation_id, content, created_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(content)
    .bind(&now)
    .execute(pool)
    .await
    .context("failed to save summary")?;

    Ok(Summary {
        id,
        conversation_id: conversation_id.to_string(),
        content: content.to_string(),
        created_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_segments(
        speakers: &[&str],
        texts: &[&str],
        durations: &[(f64, f64)],
    ) -> Vec<TranscriptionSegment> {
        speakers
            .iter()
            .zip(texts.iter())
            .zip(durations.iter())
            .map(|((speaker, text), (start, end))| TranscriptionSegment {
                speaker: speaker.to_string(),
                text: text.to_string(),
                start_time: *start,
                end_time: *end,
                confidence: 0.9,
            })
            .collect()
    }

    #[test]
    fn test_generate_summary_empty() {
        let summary = generate_summary(&[]);
        assert_eq!(summary, "No speech detected in this recording.");
    }

    #[test]
    fn test_generate_summary_single_speaker() {
        let segments = make_segments(
            &["Speaker 1", "Speaker 1"],
            &[
                "Hello, how are you doing today.",
                "I'm working on a project.",
            ],
            &[(0.0, 3.0), (3.0, 6.0)],
        );

        let summary = generate_summary(&segments);
        assert!(summary.contains("Recording spanning"));
        assert!(summary.contains("6s"));
    }

    #[test]
    fn test_generate_summary_multiple_speakers() {
        let segments = make_segments(
            &["Speaker 1", "Speaker 2", "Speaker 1"],
            &[
                "Welcome to the meeting everyone.",
                "Thanks for having me here today.",
                "Let's discuss the quarterly results.",
            ],
            &[(0.0, 3.0), (3.0, 6.0), (6.0, 65.0)],
        );

        let summary = generate_summary(&segments);
        assert!(summary.contains("2 speakers"));
        assert!(summary.contains("Key points:"));
    }

    #[test]
    fn test_generate_summary_key_points() {
        let segments = make_segments(
            &["Speaker 1"],
            &["First point about something important. Second shorter. Third is the longest sentence here with many words in it for testing purposes."],
            &[(0.0, 10.0)],
        );

        let summary = generate_summary(&segments);
        assert!(summary.contains("Key points:"));
        assert!(summary.contains("- "));
    }

    #[test]
    fn test_count_speakers() {
        let segments = make_segments(
            &["A", "B", "A", "C"],
            &["a", "b", "c", "d"],
            &[(0.0, 1.0), (1.0, 2.0), (2.0, 3.0), (3.0, 4.0)],
        );
        assert_eq!(count_speakers(&segments), 3);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30.0), "30s");
        assert_eq!(format_duration(90.0), "1m 30s");
        assert_eq!(format_duration(3661.0), "1h 1m");
    }

    #[tokio::test]
    async fn test_save_summary() {
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
            "CREATE TABLE conversations (id TEXT PRIMARY KEY, session_id TEXT NOT NULL REFERENCES sessions(id), sequence_number INTEGER NOT NULL, title TEXT NOT NULL, audio_file_path TEXT NOT NULL, duration_seconds REAL NOT NULL, status TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
            "CREATE TABLE summaries (id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL UNIQUE REFERENCES conversations(id) ON DELETE CASCADE, content TEXT NOT NULL, created_at TEXT NOT NULL)",
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

        let summary = save_summary(&pool, "c1", "Test summary content")
            .await
            .unwrap();
        assert_eq!(summary.conversation_id, "c1");
        assert_eq!(summary.content, "Test summary content");
    }
}

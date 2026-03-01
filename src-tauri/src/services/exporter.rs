use anyhow::{bail, Context, Result};
use serde::Serialize;
use sqlx::SqlitePool;

/// All data needed to export a conversation.
#[derive(Debug, Serialize)]
pub struct ExportData {
    pub title: String,
    pub created_at: String,
    pub summary: Option<String>,
    pub segments: Vec<ExportSegment>,
}

#[derive(Debug, Serialize)]
pub struct ExportSegment {
    pub speaker: String,
    pub display_name: Option<String>,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
}

/// Gather all data needed to export a conversation.
pub async fn gather_export_data(pool: &SqlitePool, conversation_id: &str) -> Result<ExportData> {
    // Fetch conversation
    let conv: Option<(String, String)> = sqlx::query_as(
        "SELECT title, created_at FROM conversations WHERE id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch conversation")?;

    let (title, created_at) = conv.ok_or_else(|| {
        anyhow::anyhow!("conversation not found: {}", conversation_id)
    })?;

    // Fetch summary
    let summary: Option<(String,)> = sqlx::query_as(
        "SELECT content FROM summaries WHERE conversation_id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch summary")?;

    // Fetch transcription segments JSON
    let transcription: Option<(String,)> = sqlx::query_as(
        "SELECT segments FROM transcriptions WHERE conversation_id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch transcription")?;

    // Fetch speaker roles
    let roles: Vec<(String, String)> = sqlx::query_as(
        "SELECT speaker_label, display_name FROM speaker_roles WHERE conversation_id = ?",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .context("failed to fetch speaker roles")?;

    let role_map: std::collections::HashMap<String, String> = roles.into_iter().collect();

    // Parse transcription segments
    let segments = if let Some((segments_json,)) = transcription {
        let raw: Vec<RawSegment> = serde_json::from_str(&segments_json)
            .context("failed to parse transcription segments")?;
        raw.into_iter()
            .map(|s| ExportSegment {
                display_name: role_map.get(&s.speaker).cloned(),
                speaker: s.speaker,
                text: s.text,
                start_time: s.start_time,
                end_time: s.end_time,
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(ExportData {
        title,
        created_at,
        summary: summary.map(|(s,)| s),
        segments,
    })
}

#[derive(serde::Deserialize)]
struct RawSegment {
    speaker: String,
    text: String,
    start_time: f64,
    end_time: f64,
}

/// Format seconds as HH:MM:SS or MM:SS.
fn format_timestamp(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

/// Format export data as a Markdown string.
pub fn format_markdown(data: &ExportData) -> String {
    let mut md = String::new();

    md.push_str(&format!("# {}\n\n", data.title));
    md.push_str(&format!("**Date**: {}\n\n", data.created_at));

    if let Some(ref summary) = data.summary {
        md.push_str("## Summary\n\n");
        md.push_str(summary);
        md.push_str("\n\n");
    }

    if !data.segments.is_empty() {
        md.push_str("## Transcript\n\n");

        for segment in &data.segments {
            let speaker = segment
                .display_name
                .as_deref()
                .unwrap_or(&segment.speaker);
            let timestamp = format_timestamp(segment.start_time);
            md.push_str(&format!(
                "**[{}] {}**: {}\n\n",
                timestamp, speaker, segment.text
            ));
        }
    }

    md
}

/// Write export data as Markdown to a file.
pub async fn export_to_markdown(
    pool: &SqlitePool,
    conversation_id: &str,
    output_path: &str,
) -> Result<()> {
    let data = gather_export_data(pool, conversation_id).await?;
    let markdown = format_markdown(&data);
    tokio::fs::write(output_path, markdown)
        .await
        .with_context(|| format!("failed to write markdown to {}", output_path))?;
    Ok(())
}

/// Gather export data as a structured JSON payload for frontend pdfmake rendering.
pub async fn export_to_pdf_payload(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<ExportData> {
    gather_export_data(pool, conversation_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp_minutes() {
        assert_eq!(format_timestamp(0.0), "00:00");
        assert_eq!(format_timestamp(65.0), "01:05");
        assert_eq!(format_timestamp(599.0), "09:59");
    }

    #[test]
    fn test_format_timestamp_hours() {
        assert_eq!(format_timestamp(3600.0), "01:00:00");
        assert_eq!(format_timestamp(3661.0), "01:01:01");
    }

    #[test]
    fn test_format_markdown_basic() {
        let data = ExportData {
            title: "Test Conversation".to_string(),
            created_at: "2026-03-01T12:00:00Z".to_string(),
            summary: Some("A test summary.".to_string()),
            segments: vec![
                ExportSegment {
                    speaker: "SPEAKER_00".to_string(),
                    display_name: Some("Alice".to_string()),
                    text: "Hello there.".to_string(),
                    start_time: 0.0,
                    end_time: 2.5,
                },
                ExportSegment {
                    speaker: "SPEAKER_01".to_string(),
                    display_name: None,
                    text: "Hi Alice!".to_string(),
                    start_time: 2.5,
                    end_time: 4.0,
                },
            ],
        };

        let md = format_markdown(&data);
        assert!(md.contains("# Test Conversation"));
        assert!(md.contains("**Date**: 2026-03-01T12:00:00Z"));
        assert!(md.contains("## Summary"));
        assert!(md.contains("A test summary."));
        assert!(md.contains("**[00:00] Alice**: Hello there."));
        assert!(md.contains("**[00:02] SPEAKER_01**: Hi Alice!"));
    }

    #[test]
    fn test_format_markdown_no_summary() {
        let data = ExportData {
            title: "No Summary".to_string(),
            created_at: "2026-03-01".to_string(),
            summary: None,
            segments: vec![],
        };

        let md = format_markdown(&data);
        assert!(md.contains("# No Summary"));
        assert!(!md.contains("## Summary"));
    }
}

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::conversation::{Conversation, ConversationStatus};
use crate::models::summary::Summary;
use crate::models::transcription::{SpeakerRole, Transcription, TranscriptionSegment};

/// List conversations in a session, ordered by sequence number.
pub async fn list_conversations(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<Conversation>> {
    let rows: Vec<(String, String, i64, String, String, f64, String, String, String)> =
        sqlx::query_as(
            "SELECT id, session_id, sequence_number, title, audio_file_path,
                    duration_seconds, status, created_at, updated_at
             FROM conversations
             WHERE session_id = ?
             ORDER BY sequence_number ASC",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
        .context("failed to list conversations")?;

    rows.into_iter()
        .map(
            |(id, session_id, sequence_number, title, audio_file_path, duration_seconds, status, created_at, updated_at)| {
                Ok(Conversation {
                    id,
                    session_id,
                    sequence_number,
                    title,
                    audio_file_path,
                    duration_seconds,
                    status: status.parse::<ConversationStatus>().map_err(|e| anyhow::anyhow!(e))?,
                    created_at,
                    updated_at,
                })
            },
        )
        .collect()
}

/// Conversation detail with optional transcription, summary, and speaker roles.
pub struct ConversationDetail {
    pub conversation: Conversation,
    pub transcription: Option<Transcription>,
    pub summary: Option<Summary>,
    pub speaker_roles: Vec<SpeakerRole>,
}

/// Get full conversation detail including transcription, summary, and speaker roles.
pub async fn get_conversation_detail(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Option<ConversationDetail>> {
    // Fetch conversation
    let conv_row: Option<(String, String, i64, String, String, f64, String, String, String)> =
        sqlx::query_as(
            "SELECT id, session_id, sequence_number, title, audio_file_path,
                    duration_seconds, status, created_at, updated_at
             FROM conversations WHERE id = ?",
        )
        .bind(conversation_id)
        .fetch_optional(pool)
        .await
        .context("failed to fetch conversation")?;

    let conv_row = match conv_row {
        Some(r) => r,
        None => return Ok(None),
    };

    let conversation = Conversation {
        id: conv_row.0,
        session_id: conv_row.1,
        sequence_number: conv_row.2,
        title: conv_row.3,
        audio_file_path: conv_row.4,
        duration_seconds: conv_row.5,
        status: conv_row.6.parse::<ConversationStatus>().map_err(|e| anyhow::anyhow!(e))?,
        created_at: conv_row.7,
        updated_at: conv_row.8,
    };

    // Fetch transcription
    let trans_row: Option<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, conversation_id, full_text, segments, created_at
         FROM transcriptions WHERE conversation_id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch transcription")?;

    let transcription = match trans_row {
        Some((id, conv_id, full_text, segments_json, created_at)) => {
            let segments: Vec<TranscriptionSegment> =
                serde_json::from_str(&segments_json).unwrap_or_default();
            Some(Transcription {
                id,
                conversation_id: conv_id,
                full_text,
                segments,
                created_at,
            })
        }
        None => None,
    };

    // Fetch summary
    let summary_row: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT id, conversation_id, content, created_at
         FROM summaries WHERE conversation_id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch summary")?;

    let summary = summary_row.map(|(id, conv_id, content, created_at)| Summary {
        id,
        conversation_id: conv_id,
        content,
        created_at,
    });

    // Fetch speaker roles
    let role_rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT id, conversation_id, speaker_label, display_name
         FROM speaker_roles WHERE conversation_id = ?",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .context("failed to fetch speaker roles")?;

    let speaker_roles = role_rows
        .into_iter()
        .map(|(id, conv_id, speaker_label, display_name)| SpeakerRole {
            id,
            conversation_id: conv_id,
            speaker_label,
            display_name,
        })
        .collect();

    Ok(Some(ConversationDetail {
        conversation,
        transcription,
        summary,
        speaker_roles,
    }))
}

/// Rename a conversation.
pub async fn rename_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
    title: &str,
) -> Result<bool> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "UPDATE conversations SET title = ?, updated_at = ? WHERE id = ?",
    )
    .bind(title)
    .bind(&now)
    .bind(conversation_id)
    .execute(pool)
    .await
    .context("failed to rename conversation")?;

    Ok(result.rows_affected() > 0)
}

/// Delete a conversation by ID. Cascade handles transcription, summary, speaker_roles.
/// Returns the audio_file_path if the conversation existed (for caller to delete the file).
pub async fn delete_conversation(
    pool: &SqlitePool,
    conversation_id: &str,
) -> Result<Option<String>> {
    // Fetch audio path before deleting
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT audio_file_path FROM conversations WHERE id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch conversation for deletion")?;

    let audio_path = match row {
        Some((path,)) => path,
        None => return Ok(None),
    };

    sqlx::query("DELETE FROM conversations WHERE id = ?")
        .bind(conversation_id)
        .execute(pool)
        .await
        .context("failed to delete conversation")?;

    Ok(Some(audio_path))
}

/// Upsert a speaker role (insert or update display name).
pub async fn upsert_speaker_role(
    pool: &SqlitePool,
    conversation_id: &str,
    speaker_label: &str,
    display_name: &str,
) -> Result<()> {
    // Check if a role already exists for this conversation + speaker_label
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM speaker_roles WHERE conversation_id = ? AND speaker_label = ?",
    )
    .bind(conversation_id)
    .bind(speaker_label)
    .fetch_optional(pool)
    .await
    .context("failed to check existing speaker role")?;

    match existing {
        Some((id,)) => {
            sqlx::query("UPDATE speaker_roles SET display_name = ? WHERE id = ?")
                .bind(display_name)
                .bind(&id)
                .execute(pool)
                .await
                .context("failed to update speaker role")?;
        }
        None => {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO speaker_roles (id, conversation_id, speaker_label, display_name)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(conversation_id)
            .bind(speaker_label)
            .bind(display_name)
            .execute(pool)
            .await
            .context("failed to insert speaker role")?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .unwrap();

        for sql in &[
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                sequence_number INTEGER NOT NULL,
                title TEXT NOT NULL,
                audio_file_path TEXT NOT NULL,
                duration_seconds REAL NOT NULL,
                status TEXT NOT NULL DEFAULT 'uploading',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL UNIQUE REFERENCES conversations(id) ON DELETE CASCADE,
                full_text TEXT NOT NULL,
                segments TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS summaries (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL UNIQUE REFERENCES conversations(id) ON DELETE CASCADE,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS speaker_roles (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                speaker_label TEXT NOT NULL,
                display_name TEXT NOT NULL
            )",
        ] {
            sqlx::query(sql).execute(&pool).await.unwrap();
        }

        pool
    }

    async fn insert_session(pool: &SqlitePool, id: &str) {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind("Test Session")
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn insert_conversation(pool: &SqlitePool, id: &str, session_id: &str, seq: i64) {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO conversations (id, session_id, sequence_number, title, audio_file_path, duration_seconds, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(session_id)
        .bind(seq)
        .bind(format!("Conversation {}", seq))
        .bind(format!("/audio/{}.wav", id))
        .bind(120.0)
        .bind("completed")
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_list_conversations() {
        let pool = setup_pool().await;
        insert_session(&pool, "s1").await;
        insert_conversation(&pool, "c1", "s1", 1).await;
        insert_conversation(&pool, "c2", "s1", 2).await;

        let convos = list_conversations(&pool, "s1").await.unwrap();
        assert_eq!(convos.len(), 2);
        assert_eq!(convos[0].sequence_number, 1);
        assert_eq!(convos[1].sequence_number, 2);
    }

    #[tokio::test]
    async fn test_list_conversations_empty_session() {
        let pool = setup_pool().await;
        insert_session(&pool, "s1").await;

        let convos = list_conversations(&pool, "s1").await.unwrap();
        assert!(convos.is_empty());
    }

    #[tokio::test]
    async fn test_get_conversation_detail() {
        let pool = setup_pool().await;
        insert_session(&pool, "s1").await;
        insert_conversation(&pool, "c1", "s1", 1).await;

        let now = Utc::now().to_rfc3339();

        // Add transcription
        let segments = serde_json::to_string(&vec![TranscriptionSegment {
            speaker: "Speaker 1".to_string(),
            text: "Hello".to_string(),
            start_time: 0.0,
            end_time: 1.5,
            confidence: 0.95,
        }])
        .unwrap();

        sqlx::query(
            "INSERT INTO transcriptions (id, conversation_id, full_text, segments, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind("t1")
        .bind("c1")
        .bind("Hello")
        .bind(&segments)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

        // Add summary
        sqlx::query(
            "INSERT INTO summaries (id, conversation_id, content, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind("sum1")
        .bind("c1")
        .bind("A greeting")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

        // Add speaker role
        sqlx::query(
            "INSERT INTO speaker_roles (id, conversation_id, speaker_label, display_name) VALUES (?, ?, ?, ?)",
        )
        .bind("sr1")
        .bind("c1")
        .bind("Speaker 1")
        .bind("Alice")
        .execute(&pool)
        .await
        .unwrap();

        let detail = get_conversation_detail(&pool, "c1").await.unwrap().unwrap();
        assert_eq!(detail.conversation.id, "c1");
        assert!(detail.transcription.is_some());
        assert_eq!(detail.transcription.as_ref().unwrap().segments.len(), 1);
        assert!(detail.summary.is_some());
        assert_eq!(detail.summary.as_ref().unwrap().content, "A greeting");
        assert_eq!(detail.speaker_roles.len(), 1);
        assert_eq!(detail.speaker_roles[0].display_name, "Alice");
    }

    #[tokio::test]
    async fn test_get_conversation_detail_not_found() {
        let pool = setup_pool().await;

        let detail = get_conversation_detail(&pool, "nonexistent").await.unwrap();
        assert!(detail.is_none());
    }

    #[tokio::test]
    async fn test_get_conversation_detail_no_transcription() {
        let pool = setup_pool().await;
        insert_session(&pool, "s1").await;
        insert_conversation(&pool, "c1", "s1", 1).await;

        let detail = get_conversation_detail(&pool, "c1").await.unwrap().unwrap();
        assert!(detail.transcription.is_none());
        assert!(detail.summary.is_none());
        assert!(detail.speaker_roles.is_empty());
    }

    #[tokio::test]
    async fn test_rename_conversation() {
        let pool = setup_pool().await;
        insert_session(&pool, "s1").await;
        insert_conversation(&pool, "c1", "s1", 1).await;

        let updated = rename_conversation(&pool, "c1", "New Title").await.unwrap();
        assert!(updated);

        let detail = get_conversation_detail(&pool, "c1").await.unwrap().unwrap();
        assert_eq!(detail.conversation.title, "New Title");
    }

    #[tokio::test]
    async fn test_rename_conversation_not_found() {
        let pool = setup_pool().await;

        let updated = rename_conversation(&pool, "fake", "Title").await.unwrap();
        assert!(!updated);
    }

    #[tokio::test]
    async fn test_delete_conversation() {
        let pool = setup_pool().await;
        insert_session(&pool, "s1").await;
        insert_conversation(&pool, "c1", "s1", 1).await;

        let path = delete_conversation(&pool, "c1").await.unwrap();
        assert!(path.is_some());
        assert!(path.unwrap().contains("c1"));

        let detail = get_conversation_detail(&pool, "c1").await.unwrap();
        assert!(detail.is_none());
    }

    #[tokio::test]
    async fn test_delete_conversation_not_found() {
        let pool = setup_pool().await;

        let path = delete_conversation(&pool, "fake").await.unwrap();
        assert!(path.is_none());
    }

    #[tokio::test]
    async fn test_delete_conversation_cascades() {
        let pool = setup_pool().await;
        insert_session(&pool, "s1").await;
        insert_conversation(&pool, "c1", "s1", 1).await;

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO transcriptions (id, conversation_id, full_text, segments, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("t1").bind("c1").bind("text").bind("[]").bind(&now)
        .execute(&pool).await.unwrap();

        sqlx::query(
            "INSERT INTO summaries (id, conversation_id, content, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind("sum1").bind("c1").bind("summary").bind(&now)
        .execute(&pool).await.unwrap();

        sqlx::query(
            "INSERT INTO speaker_roles (id, conversation_id, speaker_label, display_name) VALUES (?, ?, ?, ?)",
        )
        .bind("sr1").bind("c1").bind("Speaker 1").bind("Alice")
        .execute(&pool).await.unwrap();

        delete_conversation(&pool, "c1").await.unwrap();

        // Verify cascade
        let t: Option<(String,)> = sqlx::query_as("SELECT id FROM transcriptions WHERE conversation_id = ?")
            .bind("c1").fetch_optional(&pool).await.unwrap();
        assert!(t.is_none());

        let s: Option<(String,)> = sqlx::query_as("SELECT id FROM summaries WHERE conversation_id = ?")
            .bind("c1").fetch_optional(&pool).await.unwrap();
        assert!(s.is_none());

        let sr: Vec<(String,)> = sqlx::query_as("SELECT id FROM speaker_roles WHERE conversation_id = ?")
            .bind("c1").fetch_all(&pool).await.unwrap();
        assert!(sr.is_empty());
    }

    #[tokio::test]
    async fn test_upsert_speaker_role_insert() {
        let pool = setup_pool().await;
        insert_session(&pool, "s1").await;
        insert_conversation(&pool, "c1", "s1", 1).await;

        upsert_speaker_role(&pool, "c1", "Speaker 1", "Alice").await.unwrap();

        let roles: Vec<(String, String)> = sqlx::query_as(
            "SELECT speaker_label, display_name FROM speaker_roles WHERE conversation_id = ?",
        )
        .bind("c1")
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].0, "Speaker 1");
        assert_eq!(roles[0].1, "Alice");
    }

    #[tokio::test]
    async fn test_upsert_speaker_role_update() {
        let pool = setup_pool().await;
        insert_session(&pool, "s1").await;
        insert_conversation(&pool, "c1", "s1", 1).await;

        upsert_speaker_role(&pool, "c1", "Speaker 1", "Alice").await.unwrap();
        upsert_speaker_role(&pool, "c1", "Speaker 1", "Bob").await.unwrap();

        let roles: Vec<(String, String)> = sqlx::query_as(
            "SELECT speaker_label, display_name FROM speaker_roles WHERE conversation_id = ?",
        )
        .bind("c1")
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].1, "Bob");
    }
}

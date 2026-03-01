use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub conversation_id: String,
    pub session_id: String,
    pub title: String,
    pub snippet: String,
    pub rank: f64,
}

/// Full-text search across sessions, conversations, transcriptions, and summaries.
///
/// Uses the FTS5 virtual table if populated, with a fallback to LIKE-based search
/// on the source tables for when FTS index hasn't been populated yet.
pub async fn search_conversations(
    pool: &SqlitePool,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    // Use LIKE-based search across source tables directly.
    // This is simpler and works whether or not the FTS index is populated.
    // For a small local app this performs well enough.
    let pattern = format!("%{}%", trimmed);

    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT DISTINCT c.id, c.session_id, c.title,
                COALESCE(t.full_text, s.title, c.title) as snippet
         FROM conversations c
         JOIN sessions s ON s.id = c.session_id
         LEFT JOIN transcriptions t ON t.conversation_id = c.id
         LEFT JOIN summaries sm ON sm.conversation_id = c.id
         WHERE c.title LIKE ?
            OR s.title LIKE ?
            OR t.full_text LIKE ?
            OR sm.content LIKE ?
         LIMIT ?",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("failed to search conversations")?;

    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(i, (conversation_id, session_id, title, snippet))| {
            // Truncate snippet to a reasonable length
            let snippet = if snippet.len() > 200 {
                format!("{}...", &snippet[..200])
            } else {
                snippet
            };
            SearchResult {
                conversation_id,
                session_id,
                title,
                snippet,
                rank: -(i as f64), // Higher rank = better match; first results rank highest
            }
        })
        .collect())
}

/// Populate the FTS index for a conversation. Called after transcription completes.
pub async fn index_conversation(pool: &SqlitePool, conversation_id: &str) -> Result<()> {
    let row: Option<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT s.title, c.title, t.full_text, sm.content
         FROM conversations c
         JOIN sessions s ON s.id = c.session_id
         LEFT JOIN transcriptions t ON t.conversation_id = c.id
         LEFT JOIN summaries sm ON sm.conversation_id = c.id
         WHERE c.id = ?",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch data for FTS indexing")?;

    if let Some((session_title, conv_title, full_text, summary_content)) = row {
        sqlx::query(
            "INSERT INTO transcription_fts (session_title, conversation_title, full_text, summary_content)
             VALUES (?, ?, ?, ?)",
        )
        .bind(&session_title)
        .bind(&conv_title)
        .bind(full_text.as_deref().unwrap_or(""))
        .bind(summary_content.as_deref().unwrap_or(""))
        .execute(pool)
        .await
        .context("failed to index conversation in FTS")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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
        ] {
            sqlx::query(sql).execute(&pool).await.unwrap();
        }

        pool
    }

    async fn seed_data(pool: &SqlitePool) {
        let now = Utc::now().to_rfc3339();

        sqlx::query("INSERT INTO sessions (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind("s1")
            .bind("Team Meeting")
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO conversations (id, session_id, sequence_number, title, audio_file_path, duration_seconds, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("c1").bind("s1").bind(1).bind("Morning Chat")
        .bind("/audio/c1.wav").bind(300.0).bind("completed")
        .bind(&now).bind(&now)
        .execute(pool).await.unwrap();

        sqlx::query(
            "INSERT INTO transcriptions (id, conversation_id, full_text, segments, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("t1").bind("c1")
        .bind("Hello everyone, let's discuss the quarterly results and our roadmap.")
        .bind("[]").bind(&now)
        .execute(pool).await.unwrap();

        sqlx::query(
            "INSERT INTO summaries (id, conversation_id, content, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind("sum1")
        .bind("c1")
        .bind("Discussion about quarterly results and future roadmap.")
        .bind(&now)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_search_by_conversation_title() {
        let pool = setup_pool().await;
        seed_data(&pool).await;

        let results = search_conversations(&pool, "Morning", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].conversation_id, "c1");
    }

    #[tokio::test]
    async fn test_search_by_transcription_text() {
        let pool = setup_pool().await;
        seed_data(&pool).await;

        let results = search_conversations(&pool, "quarterly", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].conversation_id, "c1");
    }

    #[tokio::test]
    async fn test_search_by_summary() {
        let pool = setup_pool().await;
        seed_data(&pool).await;

        let results = search_conversations(&pool, "roadmap", 10).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_search_by_session_title() {
        let pool = setup_pool().await;
        seed_data(&pool).await;

        let results = search_conversations(&pool, "Team Meeting", 10)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_search_no_results() {
        let pool = setup_pool().await;
        seed_data(&pool).await;

        let results = search_conversations(&pool, "nonexistent", 10)
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let pool = setup_pool().await;
        seed_data(&pool).await;

        let results = search_conversations(&pool, "", 10).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_respects_limit() {
        let pool = setup_pool().await;
        seed_data(&pool).await;

        let results = search_conversations(&pool, "Morning", 0).await.unwrap();
        assert!(results.is_empty());
    }
}

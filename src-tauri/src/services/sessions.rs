use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::session::Session;

/// Create a new session and return it.
pub async fn create_session(
    pool: &SqlitePool,
    title: &str,
    description: Option<&str>,
) -> Result<Session> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO sessions (id, title, description, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(title)
    .bind(description)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .context("failed to insert session")?;

    Ok(Session {
        id,
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        created_at: now.clone(),
        updated_at: now,
        conversation_count: Some(0),
    })
}

/// List sessions ordered by most recent, with conversation counts.
pub async fn list_sessions(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Session>> {
    let rows: Vec<(String, String, Option<String>, String, String, i64)> = sqlx::query_as(
        "SELECT s.id, s.title, s.description, s.created_at, s.updated_at,
                COUNT(c.id) as conversation_count
         FROM sessions s
         LEFT JOIN conversations c ON c.session_id = s.id
         GROUP BY s.id
         ORDER BY s.updated_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .context("failed to list sessions")?;

    Ok(rows
        .into_iter()
        .map(|(id, title, description, created_at, updated_at, count)| Session {
            id,
            title,
            description,
            created_at,
            updated_at,
            conversation_count: Some(count),
        })
        .collect())
}

/// Get a single session by ID.
pub async fn get_session(pool: &SqlitePool, session_id: &str) -> Result<Option<Session>> {
    let row: Option<(String, String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, title, description, created_at, updated_at
         FROM sessions WHERE id = ?",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .context("failed to fetch session")?;

    Ok(row.map(|(id, title, description, created_at, updated_at)| Session {
        id,
        title,
        description,
        created_at,
        updated_at,
        conversation_count: None,
    }))
}

/// Update a session's title and/or description.
pub async fn update_session(
    pool: &SqlitePool,
    session_id: &str,
    title: Option<&str>,
    description: Option<&str>,
) -> Result<bool> {
    let now = Utc::now().to_rfc3339();

    // Build the update dynamically based on which fields are provided
    let result = match (title, description) {
        (Some(t), Some(d)) => {
            sqlx::query(
                "UPDATE sessions SET title = ?, description = ?, updated_at = ? WHERE id = ?",
            )
            .bind(t)
            .bind(d)
            .bind(&now)
            .bind(session_id)
            .execute(pool)
            .await
        }
        (Some(t), None) => {
            sqlx::query("UPDATE sessions SET title = ?, updated_at = ? WHERE id = ?")
                .bind(t)
                .bind(&now)
                .bind(session_id)
                .execute(pool)
                .await
        }
        (None, Some(d)) => {
            sqlx::query("UPDATE sessions SET description = ?, updated_at = ? WHERE id = ?")
                .bind(d)
                .bind(&now)
                .bind(session_id)
                .execute(pool)
                .await
        }
        (None, None) => {
            // Nothing to update, just touch updated_at
            sqlx::query("UPDATE sessions SET updated_at = ? WHERE id = ?")
                .bind(&now)
                .bind(session_id)
                .execute(pool)
                .await
        }
    }
    .context("failed to update session")?;

    Ok(result.rows_affected() > 0)
}

/// Delete a session by ID. Cascade deletes conversations and related records.
pub async fn delete_session(pool: &SqlitePool, session_id: &str) -> Result<bool> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(pool)
        .await
        .context("failed to delete session")?;

    Ok(result.rows_affected() > 0)
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

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
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
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_session() {
        let pool = setup_pool().await;

        let session = create_session(&pool, "Test Session", Some("A description"))
            .await
            .unwrap();

        assert_eq!(session.title, "Test Session");
        assert_eq!(session.description, Some("A description".to_string()));
        assert_eq!(session.conversation_count, Some(0));
        assert!(!session.id.is_empty());
    }

    #[tokio::test]
    async fn test_create_session_no_description() {
        let pool = setup_pool().await;

        let session = create_session(&pool, "No Desc", None).await.unwrap();

        assert_eq!(session.title, "No Desc");
        assert_eq!(session.description, None);
    }

    #[tokio::test]
    async fn test_list_sessions_ordered_by_recent() {
        let pool = setup_pool().await;

        create_session(&pool, "First", None).await.unwrap();
        create_session(&pool, "Second", None).await.unwrap();

        let sessions = list_sessions(&pool, 10, 0).await.unwrap();
        assert_eq!(sessions.len(), 2);
        // Most recent first
        assert_eq!(sessions[0].title, "Second");
        assert_eq!(sessions[1].title, "First");
    }

    #[tokio::test]
    async fn test_list_sessions_with_limit_offset() {
        let pool = setup_pool().await;

        create_session(&pool, "A", None).await.unwrap();
        create_session(&pool, "B", None).await.unwrap();
        create_session(&pool, "C", None).await.unwrap();

        let sessions = list_sessions(&pool, 1, 0).await.unwrap();
        assert_eq!(sessions.len(), 1);

        let sessions = list_sessions(&pool, 10, 2).await.unwrap();
        assert_eq!(sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_get_session() {
        let pool = setup_pool().await;

        let created = create_session(&pool, "Find Me", None).await.unwrap();
        let found = get_session(&pool, &created.id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Find Me");
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let pool = setup_pool().await;

        let found = get_session(&pool, "nonexistent-id").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_update_session_title() {
        let pool = setup_pool().await;

        let session = create_session(&pool, "Old Title", None).await.unwrap();
        let updated = update_session(&pool, &session.id, Some("New Title"), None)
            .await
            .unwrap();
        assert!(updated);

        let found = get_session(&pool, &session.id).await.unwrap().unwrap();
        assert_eq!(found.title, "New Title");
    }

    #[tokio::test]
    async fn test_update_session_description() {
        let pool = setup_pool().await;

        let session = create_session(&pool, "Title", None).await.unwrap();
        let updated = update_session(&pool, &session.id, None, Some("New desc"))
            .await
            .unwrap();
        assert!(updated);

        let found = get_session(&pool, &session.id).await.unwrap().unwrap();
        assert_eq!(found.description, Some("New desc".to_string()));
    }

    #[tokio::test]
    async fn test_update_nonexistent_session() {
        let pool = setup_pool().await;

        let updated = update_session(&pool, "fake-id", Some("Title"), None)
            .await
            .unwrap();
        assert!(!updated);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let pool = setup_pool().await;

        let session = create_session(&pool, "Delete Me", None).await.unwrap();
        let deleted = delete_session(&pool, &session.id).await.unwrap();
        assert!(deleted);

        let found = get_session(&pool, &session.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_session() {
        let pool = setup_pool().await;

        let deleted = delete_session(&pool, "fake-id").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_list_sessions_with_conversation_count() {
        let pool = setup_pool().await;

        let session = create_session(&pool, "With Convos", None).await.unwrap();

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO conversations (id, session_id, sequence_number, title, audio_file_path, duration_seconds, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("conv-1")
        .bind(&session.id)
        .bind(1)
        .bind("Convo 1")
        .bind("/audio/test.wav")
        .bind(60.0)
        .bind("completed")
        .bind(&now)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

        let sessions = list_sessions(&pool, 10, 0).await.unwrap();
        assert_eq!(sessions[0].conversation_count, Some(1));
    }
}

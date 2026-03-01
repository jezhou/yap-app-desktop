use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

/// Retrieve all settings from the database.
pub async fn get_all_settings(pool: &SqlitePool) -> Result<Vec<Setting>> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT key, value FROM settings ORDER BY key")
            .fetch_all(pool)
            .await
            .context("failed to fetch settings")?;

    Ok(rows
        .into_iter()
        .map(|(key, value)| Setting { key, value })
        .collect())
}

/// Retrieve a single setting by key.
pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(pool)
            .await
            .context("failed to fetch setting")?;

    Ok(row.map(|(value,)| value))
}

/// Insert or update a setting.
pub async fn upsert_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .context("failed to upsert setting")?;

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

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_upsert_and_get_setting() {
        let pool = setup_pool().await;

        upsert_setting(&pool, "theme", "dark").await.unwrap();
        let val = get_setting(&pool, "theme").await.unwrap();
        assert_eq!(val, Some("dark".to_string()));

        // Update existing key
        upsert_setting(&pool, "theme", "light").await.unwrap();
        let val = get_setting(&pool, "theme").await.unwrap();
        assert_eq!(val, Some("light".to_string()));
    }

    #[tokio::test]
    async fn test_get_missing_setting() {
        let pool = setup_pool().await;

        let val = get_setting(&pool, "nonexistent").await.unwrap();
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_get_all_settings() {
        let pool = setup_pool().await;

        upsert_setting(&pool, "key_a", "val_a").await.unwrap();
        upsert_setting(&pool, "key_b", "val_b").await.unwrap();

        let all = get_all_settings(&pool).await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].key, "key_a");
        assert_eq!(all[1].key, "key_b");
    }
}

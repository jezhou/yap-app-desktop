use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

mod migrations;

/// Database state managed by the Tauri app.
pub struct Database {
    pool: sqlx::SqlitePool,
}

impl Database {
    /// Initialize the database: create the file if needed, run migrations.
    pub async fn init(app_data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_data_dir)
            .context("failed to create app data directory")?;

        let db_path = app_data_dir.join("yap.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .context("failed to connect to database")?;

        // Enable WAL mode for better concurrent read performance
        sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&pool)
            .await
            .context("failed to set WAL mode")?;

        // Enable foreign keys
        sqlx::query("PRAGMA foreign_keys=ON;")
            .execute(&pool)
            .await
            .context("failed to enable foreign keys")?;

        let db = Self { pool };
        db.run_migrations().await?;

        Ok(db)
    }

    /// Run all database migrations in order.
    async fn run_migrations(&self) -> Result<()> {
        // Create migrations tracking table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await
        .context("failed to create migrations table")?;

        for (name, sql) in migrations::all() {
            let already_applied: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = ?)",
            )
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .context("failed to check migration status")?;

            if !already_applied {
                // Execute migration SQL (may contain multiple statements)
                for statement in sql.split(';') {
                    let trimmed = statement.trim();
                    if !trimmed.is_empty() {
                        sqlx::query(trimmed)
                            .execute(&self.pool)
                            .await
                            .with_context(|| {
                                format!("failed to run migration '{}': {}", name, trimmed)
                            })?;
                    }
                }

                sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
                    .bind(name)
                    .execute(&self.pool)
                    .await
                    .with_context(|| {
                        format!("failed to record migration '{}'", name)
                    })?;
            }
        }

        Ok(())
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &sqlx::SqlitePool {
        &self.pool
    }
}

/// Type alias for the managed database state.
pub type DbState = Arc<Mutex<Database>>;

/// Helper to get the database from Tauri app state.
pub fn get_db(app: &AppHandle) -> DbState {
    app.state::<DbState>().inner().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_init() {
        let tmp = TempDir::new().unwrap();
        let db = Database::init(tmp.path().to_path_buf()).await.unwrap();

        // Verify tables exist by querying sqlite_master
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
        )
        .fetch_all(db.pool())
        .await
        .unwrap();

        let table_names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();
        assert!(table_names.contains(&"sessions"), "sessions table missing");
        assert!(table_names.contains(&"conversations"), "conversations table missing");
        assert!(table_names.contains(&"transcriptions"), "transcriptions table missing");
        assert!(table_names.contains(&"summaries"), "summaries table missing");
        assert!(table_names.contains(&"speaker_roles"), "speaker_roles table missing");
        assert!(table_names.contains(&"settings"), "settings table missing");
    }

    #[tokio::test]
    async fn test_migrations_idempotent() {
        let tmp = TempDir::new().unwrap();
        let db = Database::init(tmp.path().to_path_buf()).await.unwrap();
        // Run migrations again - should not fail
        db.run_migrations().await.unwrap();
    }

    #[tokio::test]
    async fn test_foreign_keys_enabled() {
        let tmp = TempDir::new().unwrap();
        let db = Database::init(tmp.path().to_path_buf()).await.unwrap();

        let fk: (i64,) = sqlx::query_as("PRAGMA foreign_keys")
            .fetch_one(db.pool())
            .await
            .unwrap();
        assert_eq!(fk.0, 1, "foreign keys should be enabled");
    }
}

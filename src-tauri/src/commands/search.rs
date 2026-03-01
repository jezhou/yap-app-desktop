use std::sync::Arc;

use serde_json::Value;
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::search as search_service;

#[tauri::command]
pub async fn search_conversations(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    query: String,
    limit: Option<i64>,
) -> Result<Value, String> {
    let db = db.lock().await;
    let results = search_service::search_conversations(
        db.pool(),
        &query,
        limit.unwrap_or(20),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
}

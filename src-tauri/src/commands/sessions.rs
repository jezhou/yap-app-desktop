use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::sessions as sessions_service;

#[tauri::command]
pub async fn create_session(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    title: String,
    description: Option<String>,
) -> Result<Value, String> {
    let db = db.lock().await;
    let session = sessions_service::create_session(
        db.pool(),
        &title,
        description.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(json!({ "sessionId": session.id }))
}

#[tauri::command]
pub async fn list_sessions(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Value, String> {
    let db = db.lock().await;
    let sessions = sessions_service::list_sessions(
        db.pool(),
        limit.unwrap_or(50),
        offset.unwrap_or(0),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(serde_json::to_value(sessions).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn update_session(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    session_id: String,
    title: Option<String>,
    description: Option<String>,
) -> Result<Value, String> {
    let db = db.lock().await;
    let updated = sessions_service::update_session(
        db.pool(),
        &session_id,
        title.as_deref(),
        description.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    if !updated {
        return Err(format!("session not found: {}", session_id));
    }

    Ok(json!({ "updated": true }))
}

#[tauri::command]
pub async fn delete_session(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    session_id: String,
) -> Result<Value, String> {
    let db = db.lock().await;
    let deleted = sessions_service::delete_session(db.pool(), &session_id)
        .await
        .map_err(|e| e.to_string())?;

    if !deleted {
        return Err(format!("session not found: {}", session_id));
    }

    Ok(json!({ "deleted": true }))
}

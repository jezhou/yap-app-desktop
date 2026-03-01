use std::sync::Arc;

use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::settings as settings_service;

#[tauri::command]
pub async fn get_settings(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
) -> Result<Value, String> {
    let db = db.lock().await;
    let settings = settings_service::get_all_settings(db.pool())
        .await
        .map_err(|e| e.to_string())?;

    // Convert Vec<Setting> into a JSON object { key: value, ... }
    let mut map = serde_json::Map::new();
    for setting in settings {
        // Try to parse as JSON value; fall back to string
        let val = serde_json::from_str::<Value>(&setting.value)
            .unwrap_or_else(|_| Value::String(setting.value));
        map.insert(setting.key, val);
    }

    Ok(Value::Object(map))
}

#[tauri::command]
pub async fn update_settings(
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    key: String,
    value: Value,
) -> Result<Value, String> {
    let db = db.lock().await;
    // Serialize value to string for storage
    let value_str = match &value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };

    settings_service::upsert_setting(db.pool(), &key, &value_str)
        .await
        .map_err(|e| e.to_string())?;

    Ok(json!({ "updated": true }))
}

#[tauri::command]
pub async fn list_available_models() -> Result<Value, String> {
    // Stub: will be implemented with sherpa-rs model management
    Ok(json!([
        {
            "name": "whisper-tiny",
            "size": "75 MB",
            "downloaded": false,
            "qualityTier": "low"
        },
        {
            "name": "whisper-base",
            "size": "142 MB",
            "downloaded": false,
            "qualityTier": "medium"
        },
        {
            "name": "whisper-small",
            "size": "466 MB",
            "downloaded": false,
            "qualityTier": "high"
        }
    ]))
}

#[tauri::command]
pub async fn download_model(
    model_name: String,
) -> Result<Value, String> {
    // Stub: will be implemented with sherpa-rs model download + progress events
    Ok(json!({
        "status": "started",
        "modelName": model_name
    }))
}

#[tauri::command]
pub async fn delete_model(
    model_name: String,
) -> Result<Value, String> {
    // Stub: will be implemented with sherpa-rs model management
    Ok(json!({
        "deleted": true
    }))
}

#[tauri::command]
pub async fn get_diarization_status() -> Result<Value, String> {
    // Stub: will be implemented with sherpa-rs diarization check
    Ok(json!({
        "segmentationReady": false,
        "embeddingReady": false
    }))
}

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{json, Value};
use tauri::Manager;
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::settings as settings_service;

#[tauri::command]
pub async fn get_settings(db: tauri::State<'_, Arc<Mutex<Database>>>) -> Result<Value, String> {
    let db = db.lock().await;
    let settings = settings_service::get_all_settings(db.pool())
        .await
        .map_err(|e| e.to_string())?;

    // Convert Vec<Setting> into a JSON object { key: value, ... }
    let mut map = serde_json::Map::new();
    for setting in settings {
        // Try to parse as JSON value; fall back to string
        let val = serde_json::from_str::<Value>(&setting.value)
            .unwrap_or(Value::String(setting.value));
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

/// Model metadata for available STT models.
struct ModelInfo {
    name: &'static str,
    size: &'static str,
    quality_tier: &'static str,
}

const AVAILABLE_MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "whisper-tiny",
        size: "75 MB",
        quality_tier: "low",
    },
    ModelInfo {
        name: "whisper-base",
        size: "142 MB",
        quality_tier: "medium",
    },
    ModelInfo {
        name: "whisper-small",
        size: "466 MB",
        quality_tier: "high",
    },
];

fn models_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("models")
}

#[tauri::command]
pub async fn list_available_models(app: tauri::AppHandle) -> Result<Value, String> {
    let models_path = models_dir(&app);
    let models: Vec<Value> = AVAILABLE_MODELS
        .iter()
        .map(|m| {
            let downloaded = models_path.join(m.name).exists();
            json!({
                "name": m.name,
                "size": m.size,
                "downloaded": downloaded,
                "qualityTier": m.quality_tier,
            })
        })
        .collect();

    Ok(Value::Array(models))
}

#[tauri::command]
pub async fn download_model(
    app: tauri::AppHandle,
    model_name: String,
) -> Result<Value, String> {
    // Validate model name
    if !AVAILABLE_MODELS.iter().any(|m| m.name == model_name) {
        return Err(format!("unknown model: {}", model_name));
    }

    let model_dir = models_dir(&app).join(&model_name);
    std::fs::create_dir_all(&model_dir)
        .map_err(|e| format!("failed to create model directory: {}", e))?;

    // Stub: actual model download via sherpa-rs will emit progress events
    // For now, create a marker file to indicate the model is "downloaded"
    let marker = model_dir.join(".downloaded");
    std::fs::write(&marker, "placeholder")
        .map_err(|e| format!("failed to write model marker: {}", e))?;

    Ok(json!({
        "status": "completed",
        "modelName": model_name,
    }))
}

#[tauri::command]
pub async fn delete_model(
    app: tauri::AppHandle,
    model_name: String,
) -> Result<Value, String> {
    let model_dir = models_dir(&app).join(&model_name);

    if model_dir.exists() {
        std::fs::remove_dir_all(&model_dir)
            .map_err(|e| format!("failed to delete model: {}", e))?;
    }

    Ok(json!({ "deleted": true }))
}

#[tauri::command]
pub async fn get_diarization_status(app: tauri::AppHandle) -> Result<Value, String> {
    let models_path = models_dir(&app);

    // Check for diarization model directories
    let segmentation_ready = models_path.join("pyannote-segmentation").exists();
    let embedding_ready = models_path.join("3dspeaker-embedding").exists();

    Ok(json!({
        "segmentationReady": segmentation_ready,
        "embeddingReady": embedding_ready,
    }))
}

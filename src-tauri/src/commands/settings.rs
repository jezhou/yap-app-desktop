use std::path::PathBuf;
use std::sync::Arc;

use futures_util::StreamExt;
use serde_json::{json, Value};
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::settings as settings_service;

#[tauri::command]
pub async fn get_settings(db: tauri::State<'_, Arc<Mutex<Database>>>) -> Result<Value, String> {
    let db = db.lock().await;
    let settings = settings_service::get_all_settings(db.pool())
        .await
        .map_err(|e| e.to_string())?;

    // Return as array of {key, value} objects to match TypeScript Settings[] type
    Ok(serde_json::to_value(settings).map_err(|e| e.to_string())?)
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

/// A file that must be downloaded for a model.
struct ModelFile {
    filename: &'static str,
    url: &'static str,
}

/// Model metadata for available STT models.
struct ModelInfo {
    name: &'static str,
    size: &'static str,
    quality_tier: &'static str,
    files: &'static [ModelFile],
}

const AVAILABLE_MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "whisper-base",
        size: "142 MB",
        quality_tier: "base",
        files: &[
            ModelFile {
                filename: "base-encoder.int8.onnx",
                url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-base/resolve/main/base-encoder.int8.onnx",
            },
            ModelFile {
                filename: "base-decoder.int8.onnx",
                url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-base/resolve/main/base-decoder.int8.onnx",
            },
            ModelFile {
                filename: "base-tokens.txt",
                url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-base/resolve/main/base-tokens.txt",
            },
        ],
    },
    ModelInfo {
        name: "whisper-small",
        size: "466 MB",
        quality_tier: "small",
        files: &[
            ModelFile {
                filename: "small-encoder.int8.onnx",
                url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-small/resolve/main/small-encoder.int8.onnx",
            },
            ModelFile {
                filename: "small-decoder.int8.onnx",
                url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-small/resolve/main/small-decoder.int8.onnx",
            },
            ModelFile {
                filename: "small-tokens.txt",
                url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-small/resolve/main/small-tokens.txt",
            },
        ],
    },
    ModelInfo {
        name: "whisper-medium",
        size: "1.5 GB",
        quality_tier: "medium",
        files: &[
            ModelFile {
                filename: "medium-encoder.int8.onnx",
                url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-medium/resolve/main/medium-encoder.int8.onnx",
            },
            ModelFile {
                filename: "medium-decoder.int8.onnx",
                url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-medium/resolve/main/medium-decoder.int8.onnx",
            },
            ModelFile {
                filename: "medium-tokens.txt",
                url: "https://huggingface.co/csukuangfj/sherpa-onnx-whisper-medium/resolve/main/medium-tokens.txt",
            },
        ],
    },
];

/// Diarization model file definitions for auto-download.
pub const DIARIZATION_SEGMENTATION_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-segmentation-models/sherpa-onnx-pyannote-segmentation-3-0.tar.bz2";
pub const DIARIZATION_EMBEDDING_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx";

pub fn models_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("models")
}

/// Check if a model directory has all required ONNX/txt files.
fn model_is_complete(model_dir: &PathBuf, model_info: &ModelInfo) -> bool {
    if !model_dir.is_dir() {
        return false;
    }
    model_info
        .files
        .iter()
        .all(|f| model_dir.join(f.filename).exists())
}

#[tauri::command]
pub async fn list_available_models(app: tauri::AppHandle) -> Result<Value, String> {
    let models_path = models_dir(&app);
    let models: Vec<Value> = AVAILABLE_MODELS
        .iter()
        .map(|m| {
            let dir = models_path.join(m.name);
            let downloaded = model_is_complete(&dir, m);
            json!({
                "name": m.name,
                "size": m.size,
                "downloaded": downloaded,
                "quality_tier": m.quality_tier,
            })
        })
        .collect();

    Ok(Value::Array(models))
}

/// Stream-download a single file from a URL to a local path with progress reporting.
/// Writes to a `.tmp` file first, then renames for atomicity.
/// Skips if the file already exists (resume support).
async fn download_file(
    client: &reqwest::Client,
    url: &str,
    dest: &PathBuf,
    on_progress: &impl Fn(u64, u64),
) -> Result<(), String> {
    // Skip if already downloaded
    if dest.exists() {
        return Ok(());
    }

    let tmp_path = dest.with_extension("tmp");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed for {}: {}", url, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP {} for {}",
            response.status(),
            url
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(|e| format!("failed to create temp file: {}", e))?;

    let mut stream = response.bytes_stream();
    use tokio::io::AsyncWriteExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("download stream error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("failed to write chunk: {}", e))?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total_size);
    }

    file.flush()
        .await
        .map_err(|e| format!("failed to flush file: {}", e))?;
    drop(file);

    // Atomic rename
    tokio::fs::rename(&tmp_path, dest)
        .await
        .map_err(|e| format!("failed to rename temp file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn download_model(app: tauri::AppHandle, model_name: String) -> Result<Value, String> {
    // Validate model name and get model info
    let model_info = AVAILABLE_MODELS
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| format!("unknown model: {}", model_name))?;

    let model_dir = models_dir(&app).join(&model_name);

    // Check if already fully downloaded
    if model_is_complete(&model_dir, model_info) {
        return Err(format!("model already downloaded: {}", model_name));
    }

    std::fs::create_dir_all(&model_dir)
        .map_err(|e| format!("failed to create model directory: {}", e))?;

    // Collect file list for the background task
    let files: Vec<(String, String)> = model_info
        .files
        .iter()
        .map(|f| (f.filename.to_string(), f.url.to_string()))
        .collect();

    let app_clone = app.clone();
    let name = model_name.clone();
    let dir = model_dir.clone();

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let total_files = files.len() as f64;

        for (i, (filename, url)) in files.iter().enumerate() {
            let dest = dir.join(filename);
            let file_base_pct = (i as f64 / total_files) * 100.0;
            let file_range = 100.0 / total_files;

            let app_ref = &app_clone;
            let name_ref = &name;

            let result = download_file(&client, url, &dest, &|downloaded, total| {
                let file_pct = if total > 0 {
                    downloaded as f64 / total as f64
                } else {
                    0.0
                };
                let overall_pct = file_base_pct + (file_pct * file_range);
                // Throttle: only emit at 1% increments
                let rounded = (overall_pct * 1.0).round();
                let _ = app_ref.emit(
                    "model-download-progress",
                    json!({
                        "modelName": name_ref,
                        "percent": rounded.min(99.0),
                    }),
                );
            })
            .await;

            if let Err(e) = result {
                eprintln!("model download error for {}: {}", filename, e);
                // Clean up partial download
                let _ = std::fs::remove_dir_all(&dir);
                let _ = app_clone.emit(
                    "model-download-error",
                    json!({
                        "modelName": name,
                        "error": e,
                    }),
                );
                return;
            }
        }

        // Final 100% progress
        let _ = app_clone.emit(
            "model-download-progress",
            json!({
                "modelName": name,
                "percent": 100.0,
            }),
        );
    });

    Ok(json!({
        "status": "started",
        "modelName": model_name,
    }))
}

#[tauri::command]
pub async fn delete_model(app: tauri::AppHandle, model_name: String) -> Result<Value, String> {
    if !AVAILABLE_MODELS.iter().any(|m| m.name == model_name) {
        return Err(format!("unknown model: {}", model_name));
    }

    let model_dir = models_dir(&app).join(&model_name);

    if !model_dir.is_dir() {
        return Err(format!("model not downloaded: {}", model_name));
    }

    std::fs::remove_dir_all(&model_dir).map_err(|e| format!("failed to delete model: {}", e))?;

    Ok(json!({ "deleted": true }))
}

#[tauri::command]
pub async fn get_diarization_status(app: tauri::AppHandle) -> Result<Value, String> {
    let models_path = models_dir(&app);

    // Check for actual .onnx model files, not just directories
    let segmentation_ready = models_path
        .join("pyannote-segmentation")
        .join("model.onnx")
        .exists();
    let embedding_ready = {
        let emb_dir = models_path.join("3dspeaker-embedding");
        emb_dir.is_dir()
            && std::fs::read_dir(&emb_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .any(|e| e.file_name().to_string_lossy().ends_with(".onnx"))
                })
                .unwrap_or(false)
    };

    Ok(json!({
        "segmentationReady": segmentation_ready,
        "embeddingReady": embedding_ready,
    }))
}

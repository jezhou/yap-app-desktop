use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::db::Database;
use crate::services::{summarizer, transcription as transcription_service};

/// Shared state for tracking active transcription cancellation tokens.
pub struct TranscriptionState {
    /// Map of conversation_id -> cancellation flag
    pub active: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl TranscriptionState {
    pub fn new() -> Self {
        Self {
            active: Mutex::new(HashMap::new()),
        }
    }
}

pub type TranscriptionStateHandle = Arc<TranscriptionState>;

#[tauri::command]
pub async fn start_transcription(
    app: AppHandle,
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    transcription_state: tauri::State<'_, TranscriptionStateHandle>,
    conversation_id: String,
) -> Result<Value, String> {
    // Look up conversation to get audio file path and validate status
    let audio_file_path = {
        let db = db.lock().await;
        let row: Option<(String, String)> =
            sqlx::query_as("SELECT audio_file_path, status FROM conversations WHERE id = ?")
                .bind(&conversation_id)
                .fetch_optional(db.pool())
                .await
                .map_err(|e| format!("database error: {}", e))?;

        let (path, status) =
            row.ok_or_else(|| format!("conversation not found: {}", conversation_id))?;

        // Only allow transcription from valid states
        if status != "uploading" && status != "error" {
            return Err(format!(
                "cannot start transcription: conversation is in '{}' state (must be 'uploading' or 'error')",
                status
            ));
        }

        path
    };

    // Resolve model directory and check that a model is downloaded
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {}", e))?;
    let model_dir = app_data_dir.join("models");

    // Check if any STT model is downloaded
    {
        let db_lock = db.lock().await;

        // Check if user has a selected model in settings
        let selected_model: Option<String> =
            crate::services::settings::get_setting(db_lock.pool(), "selectedModel")
                .await
                .map_err(|e| e.to_string())?;

        let has_model = if let Some(ref model_name) = selected_model {
            // Check if the specifically selected model is downloaded
            model_dir.join(model_name).is_dir()
        } else {
            // No model selected — check if any whisper model directory exists
            model_dir.is_dir()
                && std::fs::read_dir(&model_dir)
                    .map(|entries| {
                        entries.filter_map(|e| e.ok()).any(|entry| {
                            entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                                && entry
                                    .file_name()
                                    .to_str()
                                    .map(|n| n.starts_with("whisper-"))
                                    .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
        };

        if !has_model {
            return Err(
                "ModelNotDownloaded: please download a transcription model in Settings before transcribing"
                    .to_string(),
            );
        }
    }

    // Create cancellation token
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut active = transcription_state.active.lock().await;
        active.insert(conversation_id.clone(), cancel.clone());
    }

    // Update status to analyzing
    {
        let db = db.lock().await;
        transcription_service::update_conversation_status(db.pool(), &conversation_id, "analyzing")
            .await
            .map_err(|e| e.to_string())?;
    }

    // Clone values for the async task
    let conv_id = conversation_id.clone();
    let db_state = db.inner().clone();
    let ts_state = transcription_state.inner().clone();
    let app_handle = app.clone();

    // Spawn the transcription pipeline as a background task
    tokio::spawn(async move {
        let result = run_transcription_pipeline(
            &app_handle,
            &db_state,
            &conv_id,
            &audio_file_path,
            &model_dir,
            cancel,
        )
        .await;

        // Remove from active transcriptions
        {
            let mut active = ts_state.active.lock().await;
            active.remove(&conv_id);
        }

        // Handle errors by setting status to "error"
        if let Err(e) = result {
            eprintln!("transcription failed for {}: {}", conv_id, e);
            let db = db_state.lock().await;
            let _ = transcription_service::update_conversation_status(db.pool(), &conv_id, "error")
                .await;
        }
    });

    Ok(json!({ "status": "analyzing" }))
}

/// Run the full transcription pipeline: STT + diarization + summarization.
async fn run_transcription_pipeline(
    app: &AppHandle,
    db_state: &Arc<Mutex<Database>>,
    conversation_id: &str,
    audio_file_path: &str,
    model_dir: &PathBuf,
    cancel: Arc<AtomicBool>,
) -> Result<(), String> {
    let conv_id = conversation_id.to_string();
    let app_clone = app.clone();

    // Set up progress callback that emits Tauri events
    let on_progress: transcription_service::ProgressCallback = Box::new(move |percent| {
        let _ = app_clone.emit(
            "transcription-progress",
            json!({
                "conversationId": conv_id,
                "percent": percent,
            }),
        );
    });

    // Run transcription
    let result = transcription_service::transcribe_audio(
        &PathBuf::from(audio_file_path),
        model_dir,
        cancel,
        Some(on_progress),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Save transcription to database
    {
        let db = db_state.lock().await;
        transcription_service::save_transcription(db.pool(), conversation_id, &result)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Check for "no speech detected" condition
    // Stub uses confidence -1.0 as sentinel, so == 0.0 correctly excludes it
    let no_speech = result.segments.is_empty()
        || (result.segments.len() == 1 && result.segments[0].confidence == 0.0);
    if no_speech {
        let _ = app.emit(
            "transcription-no-speech",
            json!({
                "conversationId": conversation_id,
                "message": "No speech detected in this recording."
            }),
        );
    }

    // Generate and save summary
    let summary_content = summarizer::generate_summary(&result.segments);
    {
        let db = db_state.lock().await;
        summarizer::save_summary(db.pool(), conversation_id, &summary_content)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Index for search
    {
        let db = db_state.lock().await;
        let _ = crate::services::search::index_conversation(db.pool(), conversation_id).await;
    }

    // Update status to completed
    {
        let db = db_state.lock().await;
        transcription_service::update_conversation_status(db.pool(), conversation_id, "completed")
            .await
            .map_err(|e| e.to_string())?;
    }

    // Emit final 100% progress
    let _ = app.emit(
        "transcription-progress",
        json!({
            "conversationId": conversation_id,
            "percent": 100.0,
        }),
    );

    Ok(())
}

#[tauri::command]
pub async fn cancel_transcription(
    transcription_state: tauri::State<'_, TranscriptionStateHandle>,
    db: tauri::State<'_, Arc<Mutex<Database>>>,
    conversation_id: String,
) -> Result<Value, String> {
    // Set the cancellation flag
    let cancelled = {
        let active = transcription_state.active.lock().await;
        if let Some(cancel) = active.get(&conversation_id) {
            cancel.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    };

    if !cancelled {
        return Err(format!(
            "no active transcription for conversation: {}",
            conversation_id
        ));
    }

    // Update status to error
    {
        let db = db.lock().await;
        transcription_service::update_conversation_status(db.pool(), &conversation_id, "error")
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(json!({
        "status": "error",
        "reason": "cancelled"
    }))
}

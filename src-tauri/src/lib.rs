pub mod commands;
pub mod db;
pub mod models;
pub mod services;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex;

/// Download diarization models in background if not already present.
/// Non-fatal: transcription works without diarization (single speaker fallback).
fn spawn_diarization_download(models_dir: PathBuf) {
    let seg_dir = models_dir.join("pyannote-segmentation");
    let emb_dir = models_dir.join("3dspeaker-embedding");

    let seg_model = seg_dir.join("model.onnx");
    let emb_model_exists = emb_dir.is_dir()
        && std::fs::read_dir(&emb_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.file_name().to_string_lossy().ends_with(".onnx"))
            })
            .unwrap_or(false);

    let need_seg = !seg_model.exists();
    let need_emb = !emb_model_exists;

    if !need_seg && !need_emb {
        return;
    }

    tokio::spawn(async move {
        let client = reqwest::Client::new();

        // Download pyannote segmentation model (tar.bz2 archive)
        if need_seg {
            eprintln!("auto-downloading pyannote segmentation model...");
            if let Err(e) = download_segmentation_model(&client, &seg_dir).await {
                eprintln!("failed to download segmentation model (non-fatal): {}", e);
            } else {
                eprintln!("pyannote segmentation model downloaded successfully");
            }
        }

        // Download 3dspeaker embedding model (single .onnx file)
        if need_emb {
            eprintln!("auto-downloading 3dspeaker embedding model...");
            if let Err(e) = download_embedding_model(&client, &emb_dir).await {
                eprintln!("failed to download embedding model (non-fatal): {}", e);
            } else {
                eprintln!("3dspeaker embedding model downloaded successfully");
            }
        }
    });
}

/// Download and extract pyannote segmentation model from tar.bz2 archive.
async fn download_segmentation_model(
    client: &reqwest::Client,
    dest_dir: &PathBuf,
) -> Result<(), String> {
    use futures_util::StreamExt;
    use std::io::Read;

    let url = commands::settings::DIARIZATION_SEGMENTATION_URL;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {} for {}", response.status(), url));
    }

    // Download entire archive to memory (segmentation model is ~5MB)
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("download error: {}", e))?;
        bytes.extend_from_slice(&chunk);
    }

    // Decompress bzip2 and extract tar
    let decoder = bzip2::read::BzDecoder::new(bytes.as_slice());
    let mut archive = tar::Archive::new(decoder);

    std::fs::create_dir_all(dest_dir)
        .map_err(|e| format!("failed to create dir: {}", e))?;

    // Extract model.onnx from the archive
    for entry in archive.entries().map_err(|e| format!("tar error: {}", e))? {
        let mut entry = entry.map_err(|e| format!("tar entry error: {}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("tar path error: {}", e))?
            .to_path_buf();

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if filename == "model.onnx" {
            let dest = dest_dir.join("model.onnx");
            let mut content = Vec::new();
            entry
                .read_to_end(&mut content)
                .map_err(|e| format!("read error: {}", e))?;
            std::fs::write(&dest, &content)
                .map_err(|e| format!("write error: {}", e))?;
            return Ok(());
        }
    }

    Err("model.onnx not found in archive".to_string())
}

/// Download 3dspeaker embedding model (single .onnx file).
async fn download_embedding_model(
    client: &reqwest::Client,
    dest_dir: &PathBuf,
) -> Result<(), String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let url = commands::settings::DIARIZATION_EMBEDDING_URL;
    let filename = "3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx";

    std::fs::create_dir_all(dest_dir)
        .map_err(|e| format!("failed to create dir: {}", e))?;

    let dest = dest_dir.join(filename);
    if dest.exists() {
        return Ok(());
    }

    let tmp_path = dest.with_extension("tmp");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {} for {}", response.status(), url));
    }

    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(|e| format!("failed to create file: {}", e))?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("download error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("write error: {}", e))?;
    }

    file.flush()
        .await
        .map_err(|e| format!("flush error: {}", e))?;
    drop(file);

    tokio::fs::rename(&tmp_path, &dest)
        .await
        .map_err(|e| format!("rename error: {}", e))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            // Ensure the audio files directory exists
            let audio_dir = app_data_dir.join("audio");
            std::fs::create_dir_all(&audio_dir).expect("failed to create audio directory");

            // Ensure models directory exists
            let models_dir = app_data_dir.join("models");
            std::fs::create_dir_all(&models_dir).expect("failed to create models directory");

            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            let database = rt
                .block_on(db::Database::init(app_data_dir))
                .expect("failed to initialize database");

            app.manage(Arc::new(Mutex::new(database)) as db::DbState);

            // Initialize audio player
            let audio_player: services::audio_player::AudioPlayerState =
                Arc::new(services::audio_player::AudioPlayer::new());
            app.manage(audio_player);

            // Initialize transcription state for cancellation tracking
            let transcription_state: commands::transcription::TranscriptionStateHandle =
                Arc::new(commands::transcription::TranscriptionState::new());
            app.manage(transcription_state);

            // Auto-download diarization models in background (non-fatal if fails)
            spawn_diarization_download(models_dir);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Audio
            commands::audio::upload_audio,
            commands::audio::play_audio,
            commands::audio::pause_audio,
            commands::audio::get_audio_position,
            // Transcription
            commands::transcription::start_transcription,
            commands::transcription::cancel_transcription,
            // Sessions
            commands::sessions::create_session,
            commands::sessions::list_sessions,
            commands::sessions::update_session,
            commands::sessions::delete_session,
            // Conversations
            commands::conversations::list_conversations,
            commands::conversations::get_conversation_detail,
            commands::conversations::rename_conversation,
            commands::conversations::delete_conversation,
            commands::conversations::update_speaker_role,
            // Search
            commands::search::search_conversations,
            // Export
            commands::export::export_markdown,
            commands::export::export_pdf,
            // Settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::list_available_models,
            commands::settings::download_model,
            commands::settings::delete_model,
            commands::settings::get_diarization_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

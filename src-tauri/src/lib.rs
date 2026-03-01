pub mod commands;
pub mod db;
pub mod models;
pub mod services;

use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let rt = tokio::runtime::Runtime::new()
                .expect("failed to create tokio runtime");
            let database = rt
                .block_on(db::Database::init(app_data_dir))
                .expect("failed to initialize database");

            app.manage(Arc::new(Mutex::new(database)) as db::DbState);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

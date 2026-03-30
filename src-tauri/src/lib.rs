pub mod commands;
pub mod error;
pub mod pipeline;
pub mod presets;
pub mod types;

use tauri::Manager;

fn evict_preview_cache(app: &tauri::AppHandle) {
  let Ok(cache_dir) = app.path().app_cache_dir() else {
    return;
  };

    let preview_dir: std::path::PathBuf = cache_dir.join("previews");
    let Ok(entries) = std::fs::read_dir(&preview_dir) else {
        return;
    };

    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(7 * 24 * 60 * 60))
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata() {
            if meta.modified().map(|t| t < cutoff).unwrap_or(false) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("image_pdf_app_lib=info,tauri=warn")
            }),
        )
        .without_time()
        .try_init();

    tracing::info!("starting PageStack");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            evict_preview_cache(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_images_from_folder,
            commands::load_images_from_paths,
            commands::generate_pdf
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

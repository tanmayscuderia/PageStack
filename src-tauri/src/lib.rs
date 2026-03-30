pub mod commands;
pub mod pipeline;
pub mod presets;
pub mod types;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::load_images_from_folder,
            commands::load_images_from_paths,
            commands::generate_pdf
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

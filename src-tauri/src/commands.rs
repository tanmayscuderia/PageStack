use std::fs;
use std::path::Path;

use base64::Engine;
use crate::types::{AppImage, GenerateRequest, GenerateResult};

#[tauri::command]
pub fn load_images_from_folder(folder_path: String) -> Result<Vec<AppImage>, String> {
    let mut images = Vec::new();

    for entry in fs::read_dir(&folder_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !is_supported_image(&path) {
            continue;
        }

        images.push(image_from_path(path));
    }

    images.sort_by(|a, b| natord::compare(&a.name, &b.name));
    Ok(images)
}

#[tauri::command]
pub fn load_images_from_paths(paths: Vec<String>) -> Result<Vec<AppImage>, String> {
    let mut images = Vec::new();

    for path in paths {
        let path = Path::new(&path);
        if !is_supported_image(path) {
            continue;
        }

        images.push(image_from_path(path.to_path_buf()));
    }

    Ok(images)
}

#[tauri::command]
pub fn generate_pdf(request: GenerateRequest) -> Result<GenerateResult, String> {
    crate::pipeline::generate_pdf(request).map_err(|e| e.to_string())
}

fn is_supported_image(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_ascii_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tif" | "tiff"
        ),
        None => false,
    }
}

fn image_from_path(path: std::path::PathBuf) -> AppImage {
    let size_bytes = fs::metadata(&path).ok().map(|m| m.len());
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".to_string());
    let preview_data_url = preview_data_url_for_path(&path).ok();

    AppImage {
        path: path.to_string_lossy().to_string(),
        name,
        size_bytes,
        preview_data_url,
    }
}

fn preview_data_url_for_path(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let mime = match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("tif") | Some("tiff") => "image/tiff",
        _ => "application/octet-stream",
    };

    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{};base64,{}", mime, encoded))
}

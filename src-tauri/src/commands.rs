use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use rayon::prelude::*;
use tauri::Manager;
use tracing::{info, warn};

use crate::error::AppError;
use crate::types::{AppImage, GenerateRequest, GenerateResult};

const MAX_IMAGES_PER_BATCH: usize = 250;
const MAX_IMAGE_FILE_BYTES: u64 = 50 * 1024 * 1024;
const PREVIEW_MAX_EDGE: u32 = 360;
const PREVIEW_JPEG_QUALITY: u8 = 72;

#[tauri::command]
pub fn load_images_from_folder(app: tauri::AppHandle, folder_path: String) -> Result<Vec<AppImage>, String> {
    info!(folder_path = %folder_path, "loading images from folder");
    let mut paths = Vec::new();

    for entry in fs::read_dir(&folder_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !is_supported_image(&path) {
            continue;
        }

        paths.push(path);
    }

    paths.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        let b_name = b.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        natord::compare(a_name, b_name)
    });

    load_images(&app, paths, &folder_path)
}

#[tauri::command]
pub fn load_images_from_paths(app: tauri::AppHandle, paths: Vec<String>) -> Result<Vec<AppImage>, String> {
    let paths = paths.into_iter().map(PathBuf::from).collect::<Vec<_>>();
    load_images(&app, paths, "dropped files")
}

#[tauri::command]
pub fn generate_pdf(app: tauri::AppHandle, request: GenerateRequest) -> Result<GenerateResult, String> {
    crate::pipeline::generate_pdf(app, request).map_err(|e| e.ipc_error_string())
}

fn load_images(app: &tauri::AppHandle, paths: Vec<PathBuf>, source: &str) -> Result<Vec<AppImage>, String> {
    let mut images = Vec::new();
    let mut seen = HashSet::new();
    let mut load_queue = Vec::new();

    for path in paths {
        if images.len() >= MAX_IMAGES_PER_BATCH {
            return Err(AppError::TooManyImages {
                count: images.len() + 1,
                limit: MAX_IMAGES_PER_BATCH,
            }
            .ipc_error_string());
        }

        let key = path.to_string_lossy().to_string();
        if !seen.insert(key) {
            continue;
        }

        if !is_supported_image(&path) {
            continue;
        }

        load_queue.push(path);
    }

    if load_queue.len() > MAX_IMAGES_PER_BATCH {
        return Err(AppError::TooManyImages {
            count: load_queue.len(),
            limit: MAX_IMAGES_PER_BATCH,
        }
        .ipc_error_string());
    }

    let app = app.clone();
    images = load_queue
        .into_par_iter()
        .map(|path| image_from_path(&app, path).map_err(|err| err.ipc_error_string()))
        .collect::<Result<Vec<_>, _>>()?;

    if images.is_empty() {
        warn!(source = %source, "no supported images found");
        return Err(AppError::NoSupportedImages {
            context: source.to_string(),
        }
        .ipc_error_string());
    }

    Ok(images)
}

fn is_supported_image(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    match ImageReader::open(path).and_then(|reader| reader.with_guessed_format()) {
        Ok(reader) => matches!(
            reader.format(),
            Some(
                ImageFormat::Jpeg
                    | ImageFormat::Png
                    | ImageFormat::WebP
                    | ImageFormat::Bmp
                    | ImageFormat::Tiff
            )
        ),
        Err(_) => false,
    }
}

fn image_from_path(app: &tauri::AppHandle, path: PathBuf) -> Result<AppImage, AppError> {
    let metadata = fs::metadata(&path).map_err(|_| {
        AppError::InvalidFile {
            path: path.to_string_lossy().to_string(),
        }
    })?;

    if metadata.len() > MAX_IMAGE_FILE_BYTES {
        warn!(
            path = %path.to_string_lossy(),
            size_bytes = metadata.len(),
            limit_bytes = MAX_IMAGE_FILE_BYTES,
            "image rejected because it exceeds the size limit"
        );
        return Err(AppError::FileTooLarge {
            path: path.to_string_lossy().to_string(),
            size_bytes: metadata.len(),
            limit_bytes: MAX_IMAGE_FILE_BYTES,
        });
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "image".to_string());
    let preview_path = preview_path_for_path(app, &path, metadata.len())?;

    Ok(AppImage {
        path: path.to_string_lossy().to_string(),
        name,
        size_bytes: Some(metadata.len()),
        preview_path: Some(preview_path),
        preview_data_url: None,
    })
}

fn preview_path_for_path(app: &tauri::AppHandle, path: &Path, input_size: u64) -> Result<String, AppError> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|err| AppError::CreatePreviewDirectory {
            detail: err.to_string(),
        })?;
    let preview_dir = cache_dir.join("previews");
    fs::create_dir_all(&preview_dir).map_err(|err| AppError::CreatePreviewDirectory {
        detail: err.to_string(),
    })?;

    let cache_key = preview_cache_key(path, input_size);
    let preview_file = preview_dir.join(format!("{cache_key}.jpg"));

    if preview_file.exists() {
        return Ok(preview_file.to_string_lossy().to_string());
    }

    let file = File::open(path).map_err(|_| AppError::OpenImage {
        path: path.to_string_lossy().to_string(),
    })?;
    let reader = ImageReader::new(BufReader::new(file))
        .with_guessed_format()
        .map_err(|_| AppError::UnsupportedImage {
            path: path.to_string_lossy().to_string(),
        })?;
    let image = reader.decode().map_err(|err| AppError::DecodeImage {
        path: path.to_string_lossy().to_string(),
        detail: err.to_string(),
    })?;

    let image = resize_preview(image, PREVIEW_MAX_EDGE);
    let rgb = flatten_if_needed(image).to_rgb8();
    let preview_file_handle = File::create(&preview_file).map_err(|err| AppError::WritePreview {
        detail: err.to_string(),
    })?;
    let mut writer = BufWriter::new(preview_file_handle);
    let mut encoder = JpegEncoder::new_with_quality(&mut writer, PREVIEW_JPEG_QUALITY);
    encoder
        .encode_image(&DynamicImage::ImageRgb8(rgb))
        .map_err(|err| AppError::DecodeImage {
            path: path.to_string_lossy().to_string(),
            detail: err.to_string(),
        })?;

    Ok(preview_file.to_string_lossy().to_string())
}

fn preview_cache_key(path: &Path, input_size: u64) -> String {
    let mut hasher = StableHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    input_size.hash(&mut hasher);

    if let Ok(metadata) = fs::metadata(path) {
        metadata.len().hash(&mut hasher);
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                duration.as_nanos().hash(&mut hasher);
            }
        }
    }

    format!("{:016x}", hasher.finish())
}

fn resize_preview(image: DynamicImage, max_long_edge: u32) -> DynamicImage {
    let (width, height) = image.dimensions();
    let current_long_edge = width.max(height);

    if current_long_edge <= max_long_edge {
        return image;
    }

    let ratio = max_long_edge as f32 / current_long_edge as f32;
    let new_width = (width as f32 * ratio).round() as u32;
    let new_height = (height as f32 * ratio).round() as u32;
    image.resize(new_width.max(1), new_height.max(1), FilterType::Lanczos3)
}

fn flatten_if_needed(image: DynamicImage) -> DynamicImage {
    if image.color().has_alpha() {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mut rgb = image::RgbImage::new(width, height);

        for (x, y, pixel) in rgba.enumerate_pixels() {
            let [r, g, b, a] = pixel.0;
            let alpha = a as f32 / 255.0;
            let bg = 255.0;
            let nr = (r as f32 * alpha + bg * (1.0 - alpha)) as u8;
            let ng = (g as f32 * alpha + bg * (1.0 - alpha)) as u8;
            let nb = (b as f32 * alpha + bg * (1.0 - alpha)) as u8;
            rgb.put_pixel(x, y, image::Rgb([nr, ng, nb]));
        }

        DynamicImage::ImageRgb8(rgb)
    } else {
        DynamicImage::ImageRgb8(image.to_rgb8())
    }
}

struct StableHasher(u64);

impl StableHasher {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }
}

impl Hasher for StableHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

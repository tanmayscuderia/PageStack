use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use image::DynamicImage;
use image::GenericImageView;
use image::imageops::FilterType;
use printpdf::{Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Pt, RawImage, RawImageData, RawImageFormat, XObjectTransform};
use rayon::prelude::*;
use tauri::Emitter;
use tracing::{info, warn};

use crate::error::AppError;
use crate::presets::resolve_preset;
use crate::types::{GenerateRequest, GenerateResult};

const MAX_IMAGE_FILE_BYTES: u64 = 50 * 1024 * 1024;

struct PreparedPage {
    image: RawImage,
    width_px: u32,
    height_px: u32,
    input_size: u64,
}

pub fn generate_pdf(app: tauri::AppHandle, request: GenerateRequest) -> Result<GenerateResult, AppError> {
    let settings = resolve_preset(&request.preset);
    info!(page_count = request.paths.len(), preset = ?request.preset, "generating pdf");

    let total = request.paths.len();
    let completed = Arc::new(AtomicUsize::new(0));
    let app_handle = app.clone();

    let pages: Vec<PreparedPage> = request
        .paths
        .par_iter()
        .map(|path| {
            let page = prepare_image(path, settings.max_long_edge, settings.jpeg_quality)?;
            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = app_handle.emit("pdf_progress", (done, total));
            Ok(page)
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    let total_input_bytes = pages.iter().map(|p| p.input_size).sum::<u64>();
    let mut doc = PdfDocument::new("PageStack");
    let mut pdf_pages = Vec::with_capacity(pages.len());

    for page in pages {
        let image_id = doc.add_image(&page.image);
        let (page_width_mm, page_height_mm) = if page.width_px >= page.height_px {
            (297.0, 210.0)
        } else {
            (210.0, 297.0)
        };
        let transform = fit_image_transform(
            page.width_px,
            page.height_px,
            page_width_mm,
            page_height_mm,
            settings.dpi,
        );

        let page_ops = vec![Op::UseXobject {
            id: image_id,
            transform,
        }];

        pdf_pages.push(PdfPage::new(Mm(page_width_mm), Mm(page_height_mm), page_ops));
    }

    let page_count = pdf_pages.len();
    let mut warnings = Vec::new();
    let pdf_bytes = doc
        .with_pages(pdf_pages)
        .save(&PdfSaveOptions::default(), &mut warnings);

    for warning in &warnings {
        warn!(?warning, "printpdf warning");
    }

    if let Some(parent) = Path::new(&request.output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| AppError::CreateOutputDirectory {
                detail: err.to_string(),
            })?;
        }
    }

    fs::write(&request.output_path, &pdf_bytes).map_err(|err| AppError::WritePdf {
        detail: err.to_string(),
    })?;

    info!(
        output_path = %request.output_path,
        page_count = page_count,
        input_bytes = total_input_bytes,
        output_bytes = pdf_bytes.len(),
        "pdf written successfully"
    );

    Ok(GenerateResult {
        output_path: request.output_path,
        output_bytes: pdf_bytes.len() as u64,
        input_bytes: total_input_bytes,
        page_count,
    })
}

fn prepare_image(path: &str, max_long_edge: u32, _jpeg_quality: u8) -> Result<PreparedPage, AppError> {
    let metadata = fs::metadata(path).map_err(|_| AppError::InvalidFile {
        path: path.to_string(),
    })?;

    if metadata.len() > MAX_IMAGE_FILE_BYTES {
        return Err(AppError::FileTooLarge {
            path: path.to_string(),
            size_bytes: metadata.len(),
            limit_bytes: MAX_IMAGE_FILE_BYTES,
        });
    }

    let file = fs::File::open(path).map_err(|_| AppError::OpenImage {
        path: path.to_string(),
    })?;
    let reader = BufReader::new(file);
    let image = image::ImageReader::new(reader)
        .with_guessed_format()
        .map_err(|_| AppError::UnsupportedImage {
            path: path.to_string(),
        })?
        .decode()
        .map_err(|err| AppError::DecodeImage {
            path: path.to_string(),
            detail: err.to_string(),
        })?;

    let flattened = flatten_if_needed(image);
    let resized = resize_if_needed(flattened, max_long_edge);

    let rgb8 = resized.to_rgb8();
    let width_px = rgb8.width();
    let height_px = rgb8.height();
    let image = RawImage {
        pixels: RawImageData::U8(rgb8.into_raw()),
        width: width_px as usize,
        height: height_px as usize,
        data_format: RawImageFormat::RGB8,
        tag: Vec::new(),
    };

    Ok(PreparedPage {
        image,
        width_px,
        height_px,
        input_size: metadata.len(),
    })
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

fn resize_if_needed(image: DynamicImage, max_long_edge: u32) -> DynamicImage {
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

fn fit_image_transform(width_px: u32, height_px: u32, page_width_mm: f32, page_height_mm: f32, dpi: f32) -> XObjectTransform {
    let image_width_mm = (width_px as f32 / dpi) * 25.4;
    let image_height_mm = (height_px as f32 / dpi) * 25.4;
    let scale = (page_width_mm / image_width_mm).min(page_height_mm / image_height_mm);
    let fitted_width_mm = image_width_mm * scale;
    let fitted_height_mm = image_height_mm * scale;

    let translate_x = Pt::from(Mm((page_width_mm - fitted_width_mm) / 2.0));
    let translate_y = Pt::from(Mm((page_height_mm - fitted_height_mm) / 2.0));

    XObjectTransform {
        translate_x: Some(translate_x),
        translate_y: Some(translate_y),
        rotate: None,
        scale_x: Some(scale),
        scale_y: Some(scale),
        dpi: Some(dpi),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_if_needed_keeps_small_images_unchanged() {
        let image = DynamicImage::ImageRgb8(image::RgbImage::new(400, 300));
        let resized = resize_if_needed(image, 800);
        assert_eq!(resized.dimensions(), (400, 300));
    }

    #[test]
    fn resize_if_needed_scales_large_images_down() {
        let image = DynamicImage::ImageRgb8(image::RgbImage::new(4000, 3000));
        let resized = resize_if_needed(image, 1000);
        let (width, height) = resized.dimensions();
        assert!(width <= 1000);
        assert!(height <= 1000);
    }

    #[test]
    fn fit_image_transform_uses_expected_scale() {
        let transform = fit_image_transform(2000, 1000, 297.0, 210.0, 300.0);
        assert!(transform.scale_x.unwrap_or_default() > 0.0);
    }
}

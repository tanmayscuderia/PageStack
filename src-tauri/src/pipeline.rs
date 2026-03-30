use std::fs;
use std::io::BufReader;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use image::DynamicImage;
use image::GenericImageView;
use image::imageops::FilterType;
use printpdf::{Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, Pt, RawImage, XObjectTransform};
use rayon::prelude::*;

use crate::presets::resolve_preset;
use crate::types::{GenerateRequest, GenerateResult};

struct PreparedPage {
    image: RawImage,
    width_px: u32,
    height_px: u32,
    input_size: u64,
}

pub fn generate_pdf(request: GenerateRequest) -> Result<GenerateResult> {
    let settings = resolve_preset(&request.preset);

    let pages: Vec<PreparedPage> = request
        .paths
        .par_iter()
        .map(|path| prepare_image(path, settings.max_long_edge, settings.jpeg_quality))
        .collect::<Result<Vec<_>>>()?;

    let total_input_bytes = pages.iter().map(|p| p.input_size).sum::<u64>();
    let mut doc = PdfDocument::new("Image PDF");
    let mut pdf_pages = Vec::with_capacity(pages.len());

    for page in pages {
        let image_id = doc.add_image(&page.image);
        let transform = fit_image_transform(
            page.width_px,
            page.height_px,
            210.0,
            297.0,
            settings.dpi,
        );

        let page_ops = vec![Op::UseXobject {
            id: image_id,
            transform,
        }];

        pdf_pages.push(PdfPage::new(Mm(210.0), Mm(297.0), page_ops));
    }

    let page_count = pdf_pages.len();
    let mut warnings = Vec::new();
    let pdf_bytes = doc
        .with_pages(pdf_pages)
        .save(&PdfSaveOptions::default(), &mut warnings);

    if let Some(parent) = Path::new(&request.output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).context("failed to create output directory")?;
        }
    }

    fs::write(&request.output_path, &pdf_bytes).context("failed to write output pdf")?;

    Ok(GenerateResult {
        output_path: request.output_path,
        output_bytes: pdf_bytes.len() as u64,
        input_bytes: total_input_bytes,
        page_count,
    })
}

fn prepare_image(path: &str, max_long_edge: u32, jpeg_quality: u8) -> Result<PreparedPage> {
    let input_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let file = fs::File::open(path).with_context(|| format!("failed to open image: {path}"))?;
    let reader = BufReader::new(file);
    let image = image::ImageReader::new(reader)
        .with_guessed_format()
        .context("failed to detect image format")?
        .decode()
        .with_context(|| format!("failed to decode image: {path}"))?;

    let flattened = flatten_if_needed(image);
    let resized = resize_if_needed(flattened, max_long_edge);
    let (width_px, height_px) = resized.dimensions();

    let mut bytes = Vec::new();
    let rgb8 = resized.to_rgb8();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, jpeg_quality);
    encoder
        .encode_image(&DynamicImage::ImageRgb8(rgb8))
        .context("failed to encode jpeg")?;

    let mut warnings = Vec::new();
    let image = RawImage::decode_from_bytes(&bytes, &mut warnings)
        .map_err(|e| anyhow!("failed to build raw pdf image: {e}"))?;

    Ok(PreparedPage {
        image,
        width_px,
        height_px,
        input_size,
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
    image.resize(new_width, new_height, FilterType::Lanczos3)
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

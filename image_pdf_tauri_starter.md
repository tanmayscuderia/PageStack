# Image → PDF Optimizer (Tauri Starter)

This is the starter scaffold for the desktop app.

## Verdict
We can build it **here first**, then you can paste it into VS Code and run it.

For a Tauri app, VS Code is still the right place to finish wiring dependencies, build, test, and package for Windows/macOS.

---

## 1) Project structure

```text
image-pdf-app/
  package.json
  src/
    App.tsx
    main.tsx
    styles.css
    types.ts
  src-tauri/
    Cargo.toml
    tauri.conf.json
    src/
      main.rs
      lib.rs
      commands.rs
      presets.rs
      types.rs
      pipeline.rs
```

---

## 2) package.json

```json
{
  "name": "image-pdf-app",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.3",
    "@types/react-dom": "^18.3.0",
    "typescript": "^5.6.3",
    "vite": "^5.4.10"
  }
}
```

---

## 3) src/types.ts

```ts
export type QualityPreset = "small" | "balanced" | "high";

export interface AppImage {
  path: string;
  name: string;
  sizeBytes?: number;
}

export interface GenerateRequest {
  paths: string[];
  outputPath: string;
  preset: QualityPreset;
}

export interface GenerateResult {
  outputPath: string;
  outputBytes: number;
  inputBytes: number;
  pageCount: number;
}
```

---

## 4) src/main.tsx

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

---

## 5) src/App.tsx

```tsx
import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppImage, GenerateResult, QualityPreset } from "./types";

export default function App() {
  const [images, setImages] = useState<AppImage[]>([]);
  const [preset, setPreset] = useState<QualityPreset>("balanced");
  const [outputPath, setOutputPath] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<GenerateResult | null>(null);
  const [error, setError] = useState("");

  const totalInputBytes = useMemo(
    () => images.reduce((sum, img) => sum + (img.sizeBytes ?? 0), 0),
    [images]
  );

  async function pickFolder() {
    setError("");
    try {
      const files = await invoke<AppImage[]>("pick_images_from_folder");
      setImages(files);
    } catch (e) {
      setError(String(e));
    }
  }

  async function generatePdf() {
    if (!images.length || !outputPath) {
      setError("Add images and an output path first.");
      return;
    }

    setLoading(true);
    setError("");
    setResult(null);

    try {
      const res = await invoke<GenerateResult>("generate_pdf", {
        request: {
          paths: images.map((i) => i.path),
          outputPath,
          preset
        }
      });
      setResult(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  function onDrop(event: React.DragEvent<HTMLDivElement>) {
    event.preventDefault();
    setError("");

    const files = Array.from(event.dataTransfer.files)
      .filter((f) => /\.(jpg|jpeg|png|webp|bmp|tif|tiff)$/i.test(f.name))
      .map((f) => ({
        name: f.name,
        path: (f as File & { path?: string }).path || f.name,
        sizeBytes: f.size
      }));

    setImages(files);
  }

  function moveUp(index: number) {
    if (index === 0) return;
    const next = [...images];
    [next[index - 1], next[index]] = [next[index], next[index - 1]];
    setImages(next);
  }

  function moveDown(index: number) {
    if (index === images.length - 1) return;
    const next = [...images];
    [next[index + 1], next[index]] = [next[index], next[index + 1]];
    setImages(next);
  }

  return (
    <div className="shell">
      <div className="panel">
        <h1>Image → PDF Optimizer</h1>
        <p className="sub">
          Drag images or choose a folder. The app compresses smartly and exports a small, sharp PDF.
        </p>

        <div
          className="dropzone"
          onDragOver={(e) => e.preventDefault()}
          onDrop={onDrop}
        >
          <strong>Drop images here</strong>
          <span>or use folder import below</span>
        </div>

        <div className="row">
          <button onClick={pickFolder}>Pick Folder</button>
          <input
            placeholder="Output path, e.g. C:\\docs\\output.pdf"
            value={outputPath}
            onChange={(e) => setOutputPath(e.target.value)}
          />
        </div>

        <div className="row settings">
          <label>Preset</label>
          <select value={preset} onChange={(e) => setPreset(e.target.value as QualityPreset)}>
            <option value="small">Small</option>
            <option value="balanced">Balanced</option>
            <option value="high">High Quality</option>
          </select>
          <button onClick={generatePdf} disabled={loading}>
            {loading ? "Generating..." : "Generate PDF"}
          </button>
        </div>

        {error && <div className="error">{error}</div>}

        <div className="stats">
          <div>Images: {images.length}</div>
          <div>Input size: {formatBytes(totalInputBytes)}</div>
        </div>

        <div className="list">
          {images.map((img, index) => (
            <div className="listItem" key={`${img.path}-${index}`}>
              <div>
                <div className="name">{img.name}</div>
                <div className="path">{img.path}</div>
              </div>
              <div className="actions">
                <button onClick={() => moveUp(index)}>↑</button>
                <button onClick={() => moveDown(index)}>↓</button>
              </div>
            </div>
          ))}
        </div>

        {result && (
          <div className="result">
            <h3>Done</h3>
            <div>Output: {result.outputPath}</div>
            <div>Pages: {result.pageCount}</div>
            <div>Final size: {formatBytes(result.outputBytes)}</div>
            <div>Input size: {formatBytes(result.inputBytes)}</div>
          </div>
        )}
      </div>
    </div>
  );
}

function formatBytes(bytes: number) {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(2)} ${units[unitIndex]}`;
}
```

---

## 6) src/styles.css

```css
:root {
  color-scheme: dark;
  font-family: Inter, system-ui, sans-serif;
}

body {
  margin: 0;
  background: #0b0f17;
  color: #e8eefc;
}

.shell {
  min-height: 100vh;
  display: flex;
  justify-content: center;
  padding: 32px;
}

.panel {
  width: 100%;
  max-width: 920px;
  background: #121826;
  border: 1px solid #1f2937;
  border-radius: 24px;
  padding: 24px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.35);
}

h1 {
  margin: 0 0 8px;
  font-size: 32px;
}

.sub {
  color: #9fb0d0;
  margin-bottom: 20px;
}

.dropzone {
  border: 2px dashed #3b82f6;
  border-radius: 20px;
  padding: 32px;
  text-align: center;
  display: grid;
  gap: 8px;
  margin-bottom: 16px;
}

.row {
  display: flex;
  gap: 12px;
  margin-bottom: 14px;
}

.row input,
.row select,
button {
  border-radius: 12px;
  border: 1px solid #2b3952;
  background: #0e1420;
  color: #eef4ff;
  padding: 12px 14px;
}

.row input {
  flex: 1;
}

button {
  cursor: pointer;
}

.stats,
.result,
.error {
  margin: 16px 0;
  padding: 14px;
  border-radius: 16px;
  background: #0e1420;
}

.error {
  border: 1px solid #7f1d1d;
  color: #fecaca;
}

.list {
  display: grid;
  gap: 10px;
}

.listItem {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: center;
  background: #0e1420;
  border-radius: 16px;
  padding: 14px;
}

.name {
  font-weight: 700;
}

.path {
  color: #9fb0d0;
  font-size: 12px;
  word-break: break-all;
}

.actions {
  display: flex;
  gap: 8px;
}
```

---

## 7) src-tauri/Cargo.toml

```toml
[package]
name = "image-pdf-app"
version = "0.1.0"
edition = "2021"

[lib]
name = "image_pdf_app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
rayon = "1"
image = { version = "0.25", features = ["jpeg", "png", "webp", "tiff", "bmp"] }
printpdf = { version = "0.7", default-features = false, features = ["jpeg", "png"] }
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
```

---

## 8) src-tauri/src/types.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppImage {
    pub path: String,
    pub name: String,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QualityPreset {
    Small,
    Balanced,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub paths: Vec<String>,
    pub output_path: String,
    pub preset: QualityPreset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    pub output_path: String,
    pub output_bytes: u64,
    pub input_bytes: u64,
    pub page_count: usize,
}
```

---

## 9) src-tauri/src/presets.rs

```rust
use crate::types::QualityPreset;

pub struct CompressionSettings {
    pub max_long_edge: u32,
    pub jpeg_quality: u8,
    pub dpi: f32,
}

pub fn resolve_preset(preset: &QualityPreset) -> CompressionSettings {
    match preset {
        QualityPreset::Small => CompressionSettings {
            max_long_edge: 1600,
            jpeg_quality: 78,
            dpi: 130.0,
        },
        QualityPreset::Balanced => CompressionSettings {
            max_long_edge: 2200,
            jpeg_quality: 85,
            dpi: 150.0,
        },
        QualityPreset::High => CompressionSettings {
            max_long_edge: 3000,
            jpeg_quality: 90,
            dpi: 200.0,
        },
    }
}
```

---

## 10) src-tauri/src/pipeline.rs

```rust
use std::fs;
use std::io::Cursor;
use std::path::Path;

use anyhow::{Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat, RgbImage};
use printpdf::{Image as PdfImage, ImageTransform, Mm, PdfDocument};
use rayon::prelude::*;

use crate::presets::resolve_preset;
use crate::types::{GenerateRequest, GenerateResult};

struct PreparedPage {
    jpeg_bytes: Vec<u8>,
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

    let (doc, first_page, first_layer) = PdfDocument::new(
        "Image PDF",
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );

    let mut current_layer = doc.get_page(first_page).get_layer(first_layer);

    for (index, page) in pages.iter().enumerate() {
        if index > 0 {
            let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            current_layer = doc.get_page(new_page).get_layer(new_layer);
        }

        let mut warnings = Vec::new();
        let raw = printpdf::image_crate::codecs::jpeg::JpegDecoder::new(Cursor::new(&page.jpeg_bytes))
            .context("failed to decode prepared jpeg")?;
        let image = PdfImage::try_from(raw, &mut warnings).context("failed to create pdf image")?;

        let width_mm = (page.width_px as f32 / settings.dpi) * 25.4;
        let height_mm = (page.height_px as f32 / settings.dpi) * 25.4;

        image.add_to_layer(
            current_layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(0.0)),
                translate_y: Some(Mm(0.0)),
                rotate: None,
                scale_x: Some(210.0 / width_mm),
                scale_y: Some(297.0 / height_mm),
                dpi: Some(settings.dpi),
            },
        );
    }

    let bytes = doc.save_to_bytes().context("failed to build pdf bytes")?;
    fs::write(&request.output_path, &bytes).context("failed to write output pdf")?;

    let output_bytes = fs::metadata(&request.output_path)
        .context("failed to read output metadata")?
        .len();

    Ok(GenerateResult {
        output_path: request.output_path,
        output_bytes,
        input_bytes: total_input_bytes,
        page_count: pages.len(),
    })
}

fn prepare_image(path: &str, max_long_edge: u32, jpeg_quality: u8) -> Result<PreparedPage> {
    let input_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let image = image::open(path).with_context(|| format!("failed to open image: {path}"))?;
    let flattened = flatten_if_needed(image);
    let resized = resize_if_needed(flattened, max_long_edge);
    let (width_px, height_px) = resized.dimensions();

    let mut jpeg_bytes = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, jpeg_quality);
    encoder.encode_image(&resized).context("failed to encode jpeg")?;

    Ok(PreparedPage {
        jpeg_bytes,
        width_px,
        height_px,
        input_size,
    })
}

fn flatten_if_needed(image: DynamicImage) -> DynamicImage {
    match image.color().has_alpha() {
        true => {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            let mut rgb = RgbImage::new(width, height);
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
        }
        false => image.to_rgb8().into(),
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
```

---

## 11) src-tauri/src/commands.rs

```rust
use std::fs;
use std::path::Path;

use anyhow::Context;
use tauri_plugin_dialog::DialogExt;

use crate::pipeline;
use crate::types::{AppImage, GenerateRequest, GenerateResult};

#[tauri::command]
pub fn pick_images_from_folder(app: tauri::AppHandle) -> Result<Vec<AppImage>, String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .ok_or_else(|| "No folder selected".to_string())?;

    let folder_path = folder
        .into_path()
        .map_err(|_| "Failed to resolve folder path".to_string())?;

    let mut images = Vec::new();

    for entry in fs::read_dir(&folder_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !is_supported_image(&path) {
            continue;
        }
        let size_bytes = fs::metadata(&path).ok().map(|m| m.len());
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "image".to_string());

        images.push(AppImage {
            path: path.to_string_lossy().to_string(),
            name,
            size_bytes,
        });
    }

    images.sort_by(|a, b| natord::compare(&a.name, &b.name));
    Ok(images)
}

#[tauri::command]
pub fn generate_pdf(request: GenerateRequest) -> Result<GenerateResult, String> {
    pipeline::generate_pdf(request).map_err(|e| e.to_string())
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
```

---

## 12) src-tauri/src/lib.rs

```rust
pub mod commands;
pub mod pipeline;
pub mod presets;
pub mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::pick_images_from_folder,
            commands::generate_pdf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 13) src-tauri/src/main.rs

```rust
fn main() {
    image_pdf_app_lib::run();
}
```

---

## 14) One important fix before running
In `Cargo.toml`, add this too because `commands.rs` uses natural sorting:

```toml
natord = "1"
```

---

## 15) What this starter already does
- drag images into UI
- or pick a folder
- sort files
- reorder them
- choose compression preset
- generate one PDF
- compress images before embedding
- run fully offline
- no LLM needed

---

## 16) What still needs improvement in v2
This starter is good, but not yet battle-hardened.

Next upgrades:
- proper page-fit logic instead of rough full-page scaling
- EXIF auto-rotation
- output save dialog
- progress events from Rust to UI
- mixed page sizes
- optional Ghostscript ultra-compress mode
- packaging and signing
- better drag-and-drop path handling on macOS/Windows

---

## 17) Dark Eagle verdict
Build the real app in **VS Code**, but generate and refine the code **here**.

That is the efficient path:
- architect here
- code scaffold here
- run/debug/package in VS Code

---

## 18) Next move
Ask for the next upgrade and I’ll generate it directly into this doc:
- `make it production-ready`
- `add EXIF rotation`
- `add output save dialog`
- `add progress bar with backend events`
- `upgrade drag-drop handling`
- `give me exact VS Code setup steps`


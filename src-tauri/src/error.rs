use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct IpcError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{context}: no supported images were found")]
    NoSupportedImages { context: String },
    #[error("{path}: unsupported image format")]
    UnsupportedImage { path: String },
    #[error("{path}: file is too large ({size_bytes} bytes, max {limit_bytes} bytes)")]
    FileTooLarge {
        path: String,
        size_bytes: u64,
        limit_bytes: u64,
    },
    #[error("failed to create preview cache directory: {detail}")]
    CreatePreviewDirectory { detail: String },
    #[error("failed to write preview thumbnail: {detail}")]
    WritePreview { detail: String },
    #[error("{path}: invalid file")]
    InvalidFile { path: String },
    #[error("too many images selected ({count} items, max {limit})")]
    TooManyImages { count: usize, limit: usize },
    #[error("{path}: failed to decode image - {detail}")]
    DecodeImage { path: String, detail: String },
    #[error("failed to create output directory: {detail}")]
    CreateOutputDirectory { detail: String },
    #[error("failed to write output PDF: {detail}")]
    WritePdf { detail: String },
    #[error("failed to open image file: {path}")]
    OpenImage { path: String },
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NoSupportedImages { .. } => "NO_SUPPORTED_IMAGES",
            AppError::UnsupportedImage { .. } => "UNSUPPORTED_IMAGE",
            AppError::FileTooLarge { .. } => "FILE_TOO_LARGE",
            AppError::CreatePreviewDirectory { .. } => "CREATE_PREVIEW_DIRECTORY",
            AppError::WritePreview { .. } => "WRITE_PREVIEW",
            AppError::InvalidFile { .. } => "INVALID_FILE",
            AppError::TooManyImages { .. } => "TOO_MANY_IMAGES",
            AppError::DecodeImage { .. } => "DECODE_IMAGE",
            AppError::CreateOutputDirectory { .. } => "CREATE_OUTPUT_DIRECTORY",
            AppError::WritePdf { .. } => "WRITE_PDF",
            AppError::OpenImage { .. } => "OPEN_IMAGE",
        }
    }

    pub fn ipc_error(&self) -> IpcError {
        IpcError {
            code: self.code(),
            message: self.to_string(),
        }
    }

    pub fn ipc_error_string(&self) -> String {
        serde_json::to_string(&self.ipc_error()).unwrap_or_else(|_| self.to_string())
    }
}

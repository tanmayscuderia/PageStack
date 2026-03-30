use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppImage {
    pub path: String,
    pub name: String,
    pub size_bytes: Option<u64>,
    pub preview_path: Option<String>,
    pub preview_data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QualityPreset {
    Small,
    Balanced,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerateRequest {
    pub paths: Vec<String>,
    pub output_path: String,
    pub preset: QualityPreset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerateResult {
    pub output_path: String,
    pub output_bytes: u64,
    pub input_bytes: u64,
    pub page_count: usize,
}

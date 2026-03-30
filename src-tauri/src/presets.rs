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

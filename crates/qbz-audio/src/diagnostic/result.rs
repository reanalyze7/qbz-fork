use serde::Serialize;

/// Result returned to the frontend after a bit-depth capture.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BitDepthResult {
    pub sample_count: u64,
    pub sample_rate: u32,
    pub channels: u32,
    pub duration_secs: f64,
    pub or_mask: String,
    pub trailing_zeros: u32,
    pub effective_bits: u32,
}

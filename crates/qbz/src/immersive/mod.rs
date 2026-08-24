//! ImmersiveView visual glue.
//!
//! The Tauri background's "full" mode is Kawarp (Kawase blur + domain warp).
//! Slint/femtovg cannot run that shader directly, so this module produces the
//! same source material as Tauri's atmosphere texture: a tiny artwork color
//! field scaled up, blurred, saturated, warmed, and vignetted. The Slint layer
//! animates two copies in opposite directions to approximate the warp.

mod atmosphere;
mod color_math;
mod image_adjust;
mod palette;

pub use atmosphere::{
    atmosphere_from_tiny8, cover_tiny_samples, generate_atmosphere_image,
};
pub use palette::{dominant_cover_color, glow_color, lyrics_accent_color, spectrum_colors};

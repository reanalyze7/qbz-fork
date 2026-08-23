//! Color-extraction functions consumed by AlbumReactive/Spectrum/Lyrics/
//! playlist-card features.

mod dominant;
mod glow;
mod lyrics_accent;
mod spectrum;

pub use dominant::dominant_cover_color;
pub use glow::glow_color;
pub use lyrics_accent::lyrics_accent_color;
pub use spectrum::spectrum_colors;

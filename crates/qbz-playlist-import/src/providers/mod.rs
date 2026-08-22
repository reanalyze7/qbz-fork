//! Provider implementations

pub mod apple;
pub mod deezer;
pub mod spotify;
pub mod tidal;

mod detect;
mod dispatch;
mod resource;

pub use detect::detect_music_resource;
pub use dispatch::{detect_provider, fetch_playlist, ProviderKind};
pub use resource::{MusicProvider, MusicResource};

#[cfg(test)]
mod tests;

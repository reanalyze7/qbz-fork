//! Metadata extraction for audio files.
//!
//! Split by domain: cross-tag reading (`tags`), pure name/string parsing
//! (`naming`, `disc_folder`, `track_number`), folder-structure inference
//! (`folder_layout`, `artist_album`), the main extraction entry points
//! (`extract`), and artwork extraction/scoring (`artwork`). All logic lives
//! in `impl MetadataExtractor` blocks spread across these files — Rust
//! allows inherent impls to span multiple files within the same crate, so
//! every `Self::foo()` cross-call still resolves correctly regardless of
//! which file `foo` is defined in.

mod artist_album;
mod artwork;
mod disc_folder;
mod extract;
mod folder_layout;
mod naming;
mod tags;
mod track_number;

#[cfg(test)]
mod tests;

/// Metadata extractor using lofty
pub struct MetadataExtractor;

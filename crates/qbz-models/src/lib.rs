//! QBZ Models - Shared types, events, and traits
//!
//! This crate provides the foundation for all QBZ crates:
//! - Type definitions (Track, Album, Artist, etc.)
//! - Event definitions (CoreEvent enum)
//! - Trait definitions (FrontendAdapter)
//! - Playback types (QueueTrack, PlaybackState)
//! - Error types
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      qbz-models (Tier 0)                    │
//! │  Types, Events, Traits - No dependencies on other qbz-*    │
//! └─────────────────────────────────────────────────────────────┘
//!                              ↑
//!     ┌────────────────────────┼────────────────────────┐
//!     │                        │                        │
//! ┌───┴───┐              ┌─────┴─────┐            ┌─────┴─────┐
//! │qbz-audio│            │qbz-qobuz  │            │qbz-player │
//! │ Tier 1 │             │  Tier 1   │            │  Tier 2   │
//! └────────┘             └───────────┘            └───────────┘
//! ```
//!
//! # Usage
//!
//! ```rust
//! use qbz_models::{Track, Album, CoreEvent, FrontendAdapter};
//! ```

pub mod error;
pub mod events;
pub mod lenient;
pub mod mixtape;
pub mod playback;
pub mod source;
pub mod traits;
pub mod types;

// Re-export commonly used types at crate root
pub use error::{QbzError, QbzResult};
pub use events::CoreEvent;
pub use lenient::{parse_items_array, parse_items_lenient};
pub use playback::{PlaybackState, PlaybackStatus, QueueState, QueueTrack, RepeatMode};
pub use source::{ArtworkRef, PlaybackSource, TrackOriginTag};
pub use traits::{FrontendAdapter, LoggingAdapter, NoOpAdapter};
// `types` re-exports its own full public surface (see types/mod.rs), so this
// glob keeps every `qbz_models::TypeName` path working unchanged regardless
// of how the `types` module is internally split into submodules.
pub use types::*;

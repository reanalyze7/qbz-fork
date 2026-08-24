//! Albums tab: metadata-grouped browse (search/sort/group/filter derived
//! client-side), windowed cover artwork, multi-select.
//!
//! The Albums tab browses the metadata-grouped albums via
//! `get_albums_metadata_page`. Sort, search, group + filter are derived
//! client-side over the full-loaded set. Covers load via the source-aware
//! artwork pipeline: local files from disk.

pub(crate) mod artwork;
pub(crate) mod derive;
mod filter;
pub(crate) mod load;
pub(crate) mod map;
mod select;

pub use artwork::*;
pub use derive::*;
pub use filter::*;
pub use load::*;
pub use select::*;

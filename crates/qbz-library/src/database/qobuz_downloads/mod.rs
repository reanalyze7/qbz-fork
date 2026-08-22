//! Qobuz Downloads Integration: local_tracks rows with `source =
//! 'qobuz_download'`, representing tracks cached to disk from Qobuz
//! streaming/purchase downloads.

mod insert_direct;
mod insert_grouping;
mod query;
mod remove;

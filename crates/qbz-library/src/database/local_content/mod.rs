//! Offline Mode: Local Content Detection — matching Qobuz tracks/playlists
//! to their locally-cached counterparts (by Qobuz id or fuzzy metadata),
//! and tracking playlist-level "has local content" status.

mod batch;
mod detection;
mod playlist_status;

//! Extracting unique artist MBIDs from playlist track data.

use std::collections::HashSet;

/// Extract unique artist MBIDs from playlist tracks
///
/// This is a helper function that should be called from the Tauri command
/// to get artist MBIDs from track data.
pub fn extract_artist_mbids(
    tracks: &[(u64, Option<String>)], // (track_id, artist_mbid)
) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut mbids = Vec::new();

    for (_, mbid) in tracks {
        if let Some(id) = mbid {
            if !id.is_empty() && seen.insert(id.clone()) {
                mbids.push(id.clone());
            }
        }
    }

    mbids
}

use crate::*;

// Catalog/library toggle: look up the per-artist index (favorites this
// session) once, build cover jobs for the "In library" rows (they seed
// with empty images — Slint can't fetch network art). Split out of
// `navigate_artist` (part3.rs) to stay under the 130-line file cap.
pub(crate) fn artist_library_jobs(
    artist_id: &str,
) -> (Option<library_by_artist::ArtistLibrary>, Vec<artwork::ArtworkJob>) {
    let lib = crate::library_by_artist::get(artist_id);
    let mut lib_jobs: Vec<artwork::ArtworkJob> = Vec::new();
    if let Some(lib) = lib.as_ref() {
        for (index, t) in lib.tracks.iter().enumerate() {
            if !t.artwork_url.is_empty() {
                lib_jobs.push(artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::ArtistLibraryTrack { index },
                    url: t.artwork_url.clone(),
                });
            }
        }
        for (index, a) in lib.albums.iter().enumerate() {
            if !a.artwork_url.is_empty() {
                lib_jobs.push(artwork::ArtworkJob {
                    target: artwork::ArtworkTarget::ArtistLibraryAlbum { index },
                    url: a.artwork_url.clone(),
                });
            }
        }
    }
    (lib, lib_jobs)
}

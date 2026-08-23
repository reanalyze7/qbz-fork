//! ArtistsByLocationView controller — runs the scene discovery for a
//! source artist's location and pushes the validated artist grid into
//! `LocationViewState`. Mirrors Tauri's ArtistsByLocationView.svelte
//! (minus the in-progress event stream, which the Slint port replaces
//! with a simple loading flag).

mod load;
mod view;

pub use load::load_scene;
pub use view::{append_scene, apply_scene, artwork_jobs, reset_scene};

/// Validation page size — how many MB candidates to validate against
/// Qobuz per call. Matches the Tauri view's LIMIT.
pub const PAGE_SIZE: usize = 30;

pub struct LocationData {
    pub scene_label: String,
    pub genre_summary: String,
    pub artists: Vec<ArtistCard>,
    pub total: usize,
}

#[derive(Clone)]
pub struct ArtistCard {
    pub qobuz_id: String,
    pub name: String,
    pub genres_line: String,
    pub image_url: String,
}

pub(super) fn map_candidate(c: qbz_integrations::musicbrainz::LocationCandidate) -> ArtistCard {
    ArtistCard {
        qobuz_id: c.qobuz_id.map(|id| id.to_string()).unwrap_or_default(),
        name: c.qobuz_name.unwrap_or(c.mb_name),
        genres_line: c.genres.join(" · "),
        image_url: c.qobuz_image.unwrap_or_default(),
    }
}

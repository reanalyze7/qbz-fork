//! `Album`/`Track` -> `AlbumData`/`TrackData` mapping — the pure
//! "computation" half of the controller.

mod booklet;
mod credits;
mod meta;
mod text;
mod track;

pub(in crate::album) use text::{format_duration, lastfm_segment};
pub(in crate::album) use track::{mmss, tier};

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::{Album, Track};

use super::data::AlbumData;
use booklet::booklet_url;
use credits::build_credits;
use meta::build_meta_line;
use text::truncate_words;
use track::map_track;

/// Fetch and map a full album by id.
pub async fn load_album<A>(runtime: &Arc<AppRuntime<A>>, album_id: &str) -> Result<AlbumData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let album = runtime
        .core()
        .get_album(album_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(map_album(album))
}

fn map_album(album: Album) -> AlbumData {
    let artist = album.artist.name.clone();
    let artist_id = album.artist.id.to_string();
    let artists = build_credits(&album);
    let meta = build_meta_line(&album);

    let quality_tier = tier(album.maximum_bit_depth).to_string();
    let quality_detail = crate::quality::detail(album.maximum_bit_depth, album.maximum_sampling_rate);
    let description = album
        .description
        .as_deref()
        .map(crate::strip_html::strip_html)
        .unwrap_or_default();
    // The header description fills the full width to the right of the
    // artwork, so a longer truncation keeps it from looking like a thin
    // strip; the Read more modal still holds the complete text.
    let description_short = truncate_words(&description, 360);
    // Half the cutoff for the space-constrained layout (tracks get priority).
    let description_shorter = truncate_words(&description, 180);
    let artwork_url = album.image.best().cloned().unwrap_or_default();
    let label = album
        .label
        .as_ref()
        .map(|l| l.name.clone())
        .unwrap_or_default();
    let label_id = album
        .label
        .as_ref()
        .map(|l| l.id.to_string())
        .unwrap_or_default();
    let booklet_url = booklet_url(&album);
    let has_booklet = !booklet_url.is_empty();
    let raw_tracks: Vec<Track> = album
        .tracks
        .map(|container| container.items)
        .unwrap_or_default();
    let tracks = raw_tracks.iter().cloned().map(map_track).collect();
    let title = crate::album_map::format_album_title(&album.title, album.version.as_deref());

    AlbumData {
        id: album.id,
        title,
        artist,
        artist_id,
        artists,
        info_line: meta.full,
        meta_pre: meta.pre,
        meta_post: meta.post,
        quality_tier,
        quality_detail,
        description,
        description_short,
        description_shorter,
        artwork_url,
        label,
        label_id,
        has_booklet,
        booklet_url,
        tracks,
        raw_tracks,
    }
}

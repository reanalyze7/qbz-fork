//! `SavePayload` struct + Send-safe payload building for `save_tags`.

use qbz_library::{AlbumMetadataOverride, AlbumTagWrite, AlbumTrackUpdate, TrackMetadataOverride, TrackTagWrite};

use crate::TagTrackEdit;

use super::parse_num;

/// Everything the blocking save task needs, built + validated on the UI thread.
pub(super) struct SavePayload {
    pub(super) group_key: String,
    pub(super) album_title: String,
    pub(super) album_artist: String,
    pub(super) album_dir: String,
    pub(super) direct: bool,
    pub(super) year: Option<u32>,
    pub(super) genre_opt: Option<String>,
    pub(super) catalog_opt: Option<String>,
    pub(super) track_updates: Vec<AlbumTrackUpdate>,
    pub(super) tw_tracks: Vec<TrackTagWrite>,
    pub(super) track_overs: Vec<TrackMetadataOverride>,
    pub(super) album_over: AlbumMetadataOverride,
    pub(super) tw_album: AlbumTagWrite,
}

/// Build the write payloads from already-validated fields.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_payload(
    group_key: String,
    album_title: String,
    album_artist: String,
    album_dir: String,
    direct: bool,
    year: Option<u32>,
    genre: &str,
    catalog: &str,
    rows: &[TagTrackEdit],
) -> SavePayload {
    let artist_opt = {
        let a = album_artist.trim();
        if a.is_empty() { None } else { Some(a.to_string()) }
    };
    let genre_opt = {
        let g = genre.trim();
        if g.is_empty() { None } else { Some(g.to_string()) }
    };
    let catalog_opt = {
        let c = catalog.trim();
        if c.is_empty() { None } else { Some(c.to_string()) }
    };

    let track_updates: Vec<AlbumTrackUpdate> = rows
        .iter()
        .map(|r| AlbumTrackUpdate {
            id: r.id as i64,
            title: r.title.trim().to_string(),
            disc_number: parse_num(&r.disc_number),
            track_number: parse_num(&r.track_number),
        })
        .collect();
    let tw_tracks: Vec<TrackTagWrite> = rows
        .iter()
        .map(|r| TrackTagWrite {
            file_path: r.file_path.to_string(),
            title: r.title.trim().to_string(),
            track_number: parse_num(&r.track_number),
            disc_number: parse_num(&r.disc_number),
        })
        .collect();
    let track_overs: Vec<TrackMetadataOverride> = rows
        .iter()
        .map(|r| TrackMetadataOverride {
            file_path: r.file_path.to_string(),
            cue_start_secs: if r.cue_start_secs >= 0.0 {
                Some(r.cue_start_secs as f64)
            } else {
                None
            },
            title: Some(r.title.trim().to_string()),
            disc_number: parse_num(&r.disc_number),
            track_number: parse_num(&r.track_number),
        })
        .collect();
    let album_over = AlbumMetadataOverride {
        album_title: Some(album_title.clone()),
        album_artist: artist_opt,
        year,
        genre: genre_opt.clone(),
        catalog_number: catalog_opt.clone(),
    };
    let tw_album = AlbumTagWrite {
        album_title: album_title.clone(),
        album_artist: album_artist.clone(),
        year,
        genre: genre_opt.clone(),
        catalog_number: catalog_opt.clone(),
    };

    SavePayload {
        group_key,
        album_title,
        album_artist,
        album_dir,
        direct,
        year,
        genre_opt,
        catalog_opt,
        track_updates,
        tw_tracks,
        track_overs,
        album_over,
        tw_album,
    }
}

//! Pure row builders: `PmPlaylist`/`PmLocalPlaylist`/`FolderFull` -> the
//! Slint `PmPlaylistItem`/`PmFolderItem` structs.

use crate::folders::FolderFull;
use crate::{PmFolderItem, PmPlaylistItem};

use super::build_format::{format_duration, parse_color};
use super::types::{PmLocalPlaylist, PmPlaylist};

pub(super) fn folder_item(f: &FolderFull, count: usize) -> PmFolderItem {
    let color = parse_color(&f.icon_color);
    PmFolderItem {
        id: f.id.clone().into(),
        name: f.name.clone().into(),
        count: count as i32,
        icon_type: f.icon_type.clone().into(),
        icon_preset: f.icon_preset.clone().into(),
        icon_color: color.unwrap_or_default(),
        has_color: color.is_some(),
        is_hidden: f.is_hidden,
        custom_image: slint::Image::default(),
        has_custom_image: f.icon_type == "custom" && f.custom_image_path.is_some(),
    }
}

pub(super) fn playlist_item(p: &PmPlaylist) -> PmPlaylistItem {
    let url = |i: usize| -> slint::SharedString {
        p.cover_urls.get(i).cloned().unwrap_or_default().into()
    };
    let local_status = if p.local_count == 0 {
        "no"
    } else if p.tracks_count == 0 {
        "all_local"
    } else {
        "some_local"
    };
    let local_line = if p.local_count > 0 {
        qbz_i18n::t_args("({} local)", &[&p.local_count.to_string()])
    } else {
        String::new()
    };
    PmPlaylistItem {
        id: p.id.to_string().into(),
        name: p.name.clone().into(),
        tracks_line: { let n = p.total_count(); qbz_i18n::tf("{} track", "{} tracks", n as i64, &[&n.to_string()]).into() },
        duration_line: format_duration(p.duration).into(),
        local_line: local_line.into(),
        local_count: p.local_count as i32,
        total_count: p.total_count() as i32,
        play_count: p.play_count as i32,
        local_status: local_status.into(),
        is_favorite: p.is_favorite,
        is_hidden: p.is_hidden,
        is_local_playlist: false,
        offline_only: false,
        folder_id: p.folder_id.clone().unwrap_or_default().into(),
        cover_count: p.cover_urls.len().min(4) as i32,
        url1: url(0),
        url2: url(1),
        url3: url(2),
        url4: url(3),
        cover1: slint::Image::default(),
        cover2: slint::Image::default(),
        cover3: slint::Image::default(),
        cover4: slint::Image::default(),
    }
}

pub(super) fn local_playlist_item(p: &PmLocalPlaylist) -> PmPlaylistItem {
    PmPlaylistItem {
        id: p.id.clone().into(),
        name: p.name.clone().into(),
        tracks_line: qbz_i18n::tf("{} track", "{} tracks", p.track_count as i64, &[&p.track_count.to_string()]).into(),
        duration_line: "".into(),
        local_line: "".into(),
        local_count: 0,
        total_count: p.track_count as i32,
        play_count: 0,
        local_status: "".into(),
        is_favorite: p.is_favorite,
        is_hidden: p.is_hidden,
        is_local_playlist: true,
        offline_only: p.offline_only,
        folder_id: "".into(),
        cover_count: 0,
        url1: Default::default(),
        url2: Default::default(),
        url3: Default::default(),
        url4: Default::default(),
        cover1: slint::Image::default(),
        cover2: slint::Image::default(),
        cover3: slint::Image::default(),
        cover4: slint::Image::default(),
    }
}

//! The two `SidebarEntry` row builders (Qobuz playlist / local playlist).

use crate::SidebarEntry;

use super::{LocalSidebarPlaylist, SidebarPlaylist};

/// Build a playlist `SidebarEntry` (with its cover URLs for the
/// micro-collage). The decoded `cover*` images stay default here and are
/// filled asynchronously by the artwork pipeline (see `artwork_jobs`).
pub(super) fn playlist_entry(p: &SidebarPlaylist, indent: bool, folder_id: &str) -> SidebarEntry {
    let url = |i: usize| -> slint::SharedString {
        p.cover_urls.get(i).cloned().unwrap_or_default().into()
    };
    SidebarEntry {
        kind: "playlist".into(),
        id: p.id.to_string().into(),
        name: p.name.clone().into(),
        expanded: false,
        count: 0,
        indent,
        folder_id: folder_id.into(),
        local_kind: "".into(),
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

/// Build a LOCAL playlist row. `indent`/`folder_id` place it under a folder
/// (root row when `folder_id` is ""). A micro-collage is shown when the
/// playlist resolved >= 1 track cover; otherwise the row falls back to the
/// hard-drive glyph (`local_kind` stays set so the glyph branch still applies
/// when there are no covers).
pub(super) fn local_playlist_entry(
    p: &LocalSidebarPlaylist,
    indent: bool,
    folder_id: &str,
) -> SidebarEntry {
    let url = |i: usize| -> slint::SharedString {
        p.cover_urls.get(i).cloned().unwrap_or_default().into()
    };
    SidebarEntry {
        kind: "playlist".into(),
        id: p.id.clone().into(),
        name: p.name.clone().into(),
        expanded: false,
        count: 0,
        indent,
        folder_id: folder_id.into(),
        local_kind: if p.offline_only { "offline" } else { "local" }.into(),
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

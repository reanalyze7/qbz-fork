//! Albums tab quality/format/source filter — state read + predicate.

use crate::AppWindow;

/// Active quality/format/source filter (read once per derive from the global).
#[derive(Clone, Copy, Default)]
pub(crate) struct AlbumFilter {
    pub(crate) hires: bool,
    pub(crate) cd: bool,
    pub(crate) lossy: bool,
    pub(crate) flac: bool,
    pub(crate) alac: bool,
    pub(crate) ape: bool,
    pub(crate) wav: bool,
    pub(crate) mp3: bool,
    pub(crate) aac: bool,
    pub(crate) other: bool,
    pub(crate) local: bool,
    pub(crate) offline: bool,
}

pub(crate) fn read_album_filter(window: &AppWindow) -> AlbumFilter {
    let f = window.global::<crate::LibAlbumFilterState>();
    AlbumFilter {
        hires: f.get_hires(),
        cd: f.get_cd(),
        lossy: f.get_lossy(),
        flac: f.get_flac(),
        alac: f.get_alac(),
        ape: f.get_ape(),
        wav: f.get_wav(),
        mp3: f.get_mp3(),
        aac: f.get_aac(),
        other: f.get_other(),
        local: f.get_local(),
        offline: f.get_offline(),
    }
}

pub(crate) fn album_filter_count(f: &AlbumFilter) -> i32 {
    [
        f.hires, f.cd, f.lossy, f.flac, f.alac, f.ape, f.wav, f.mp3, f.aac, f.other, f.local,
        f.offline,
    ]
    .iter()
    .filter(|b| **b)
    .count() as i32
}

/// 1:1 with Tauri `matchesQualityFilters`: OR within each group, AND between
/// groups; an empty group passes everything.
pub(crate) fn album_matches_filters(a: &qbz_library::LocalAlbum, f: &AlbumFilter) -> bool {
    let format = a.format.to_string().to_lowercase();
    let lossless = matches!(
        format.as_str(),
        "flac" | "wav" | "aiff" | "alac" | "ape" | "dsd" | "dsf" | "dff"
    );
    let lossy = matches!(format.as_str(), "mp3" | "aac" | "m4a" | "ogg" | "opus" | "wma");
    let bit_depth = a.bit_depth.unwrap_or(16);

    let q_active = f.hires || f.cd || f.lossy;
    let passes_q = !q_active
        || (f.hires && lossless && (bit_depth >= 24 || a.sample_rate > 48000.0))
        || (f.cd && lossless && bit_depth <= 16 && a.sample_rate <= 48000.0)
        || (f.lossy && lossy);

    let fmt_active = f.flac || f.alac || f.ape || f.wav || f.mp3 || f.aac || f.other;
    let passes_f = !fmt_active
        || (f.flac && format == "flac")
        || (f.alac && (format == "alac" || format == "m4a"))
        || (f.ape && format == "ape")
        || (f.wav && (format == "wav" || format == "wave"))
        || (f.mp3 && format == "mp3")
        || (f.aac && (format == "aac" || format == "m4a"))
        || (f.other
            && !matches!(
                format.as_str(),
                "flac" | "alac" | "ape" | "wav" | "wave" | "mp3" | "aac" | "m4a"
            ));

    let s_active = f.local || f.offline;
    let src = a.source.as_str();
    let passes_s = !s_active
        || (f.local && (src == "user" || src.is_empty()))
        || (f.offline && src == "qobuz_download");

    passes_q && passes_f && passes_s
}

/// Clear all quality/format/source filters, then re-derive.
pub fn clear_album_filter(window: &AppWindow) {
    let f = window.global::<crate::LibAlbumFilterState>();
    f.set_hires(false);
    f.set_cd(false);
    f.set_lossy(false);
    f.set_flac(false);
    f.set_alac(false);
    f.set_ape(false);
    f.set_wav(false);
    f.set_mp3(false);
    f.set_aac(false);
    f.set_other(false);
    f.set_local(false);
    f.set_offline(false);
    super::derive_albums(window);
}

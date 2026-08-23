//! Shared log/summary formatting helpers.

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use qbz_playlist_import::ImportProvider;

use crate::{AppWindow, ImportLogEntry, PlaylistImportState};

/// Append one pre-formatted line to the conversion log (append-only
/// VecModel). Event-loop thread.
pub(super) fn push_log(window: &AppWindow, message: String, status: &str) {
    let state = window.global::<PlaylistImportState>();
    let log = state.get_log();
    let entry = ImportLogEntry {
        message: message.into(),
        status: status.into(),
    };
    if let Some(vec) = log.as_any().downcast_ref::<VecModel<ImportLogEntry>>() {
        vec.push(entry);
    } else {
        // First write after the .slint default literal — swap in a
        // VecModel (open()/begin_fetch() normally install one already).
        let mut entries: Vec<ImportLogEntry> = log.iter().collect();
        entries.push(entry);
        state.set_log(ModelRc::new(VecModel::from(entries)));
    }
}

pub(super) fn clear_summary(window: &AppWindow) {
    let state = window.global::<PlaylistImportState>();
    state.set_summary_playlist("".into());
    state.set_summary_matched("".into());
    state.set_summary_skipped("".into());
    state.set_summary_parts("".into());
}

/// "Split into {count} playlists (Qobuz 2000-track limit)" — used as both
/// a log line and the summary parts line, as in Tauri.
pub(super) fn parts_line(count: u32) -> String {
    qbz_i18n::t_args("Split into {} playlists (Qobuz 2000-track limit)", &[&count.to_string()])
}

/// Display names for the "Found N tracks from {provider}." log (Svelte
/// formatProvider). The enum is exhaustive, so Svelte's "Unknown" arm is
/// unreachable here.
pub(super) fn provider_display_name(provider: &ImportProvider) -> &'static str {
    match provider {
        ImportProvider::Spotify => "Spotify",
        ImportProvider::AppleMusic => "Apple Music",
        ImportProvider::Tidal => "Tidal",
        ImportProvider::Deezer => "Deezer",
    }
}

/// `toLocaleString()` twin for the matching log/status numbers
/// ("12,345"). Tauri rendered these with the user's locale; fixed en-US
/// grouping is the deliberate choice here.
pub(super) fn group_thousands(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::group_thousands;

    #[test]
    fn group_thousands_matches_to_locale_string() {
        assert_eq!(group_thousands(0), "0");
        assert_eq!(group_thousands(999), "999");
        assert_eq!(group_thousands(1000), "1,000");
        assert_eq!(group_thousands(12345), "12,345");
        assert_eq!(group_thousands(1234567), "1,234,567");
    }
}

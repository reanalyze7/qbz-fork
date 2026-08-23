//! Static build-info fields + the contributor list/model.

use slint::ModelRc;
use slint::VecModel;

use crate::{AboutContributorGroup, AboutContributorRow};

/// The real, displayed app version (workspace package version, e.g. "1.2.15").
pub fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Build date (`YYYY-MM-DD`) embedded by `build.rs`; empty if unavailable.
pub(super) fn build_date() -> &'static str {
    env!("QBZ_BUILD_DATE")
}

/// Short git commit embedded by `build.rs`; empty in offline source builds.
pub(super) fn build_commit() -> &'static str {
    env!("QBZ_BUILD_COMMIT")
}

/// Platform label for the build-info grid. This is the Slint port, so the label
/// reads "(Slint)" rather than the Tauri build's "(Tauri 2.0)".
pub(super) fn platform_label() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macOS (Slint)"
    }
    #[cfg(target_os = "windows")]
    {
        "Windows (Slint)"
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        "Linux (Slint)"
    }
}

/// The app author's GitHub handle (the single Author chip).
pub(super) const AUTHOR_HANDLE: &str = "vicrodh";

/// The contributor handles + their GitHub profile URLs. First the Tauri About
/// modal's order, then the Slint-era external-PR contributors: `hoyon`
/// (classical "work" grouping, PR #536), `mxnix` (Russian translation,
/// PR #517) and `TerminalTilt`.
pub(super) const CONTRIBUTORS: &[&str] = &[
    "vorce",
    "boxdot",
    "arminfelder",
    "afonsojramos",
    "GwendalBeaumont",
    "AdamArstall",
    "Vudgekek",
    "DoubleGate",
    "hoyon",
    "mxnix",
    "TerminalTilt",
];

/// How many contributor chips per wrap row. Slint has no flex-wrap, so the flat
/// list is pre-grouped into fixed rows (see `AboutContributorGroup`). 5 fills
/// the widened ~840px panel instead of the old 4-per-row / 3-row layout that
/// left the wider modal half-empty.
pub(super) const CONTRIBUTORS_PER_ROW: usize = 5;

/// Build the row-grouped contributor model. Avatars start blank (default image)
/// and are filled in async by `spawn_avatar_loads`.
pub(super) fn build_contributor_groups() -> Vec<AboutContributorGroup> {
    CONTRIBUTORS
        .chunks(CONTRIBUTORS_PER_ROW)
        .map(|chunk| {
            let rows: Vec<AboutContributorRow> = chunk
                .iter()
                .map(|handle| AboutContributorRow {
                    name: (*handle).into(),
                    url: format!("https://github.com/{handle}").into(),
                    avatar: slint::Image::default(),
                })
                .collect();
            AboutContributorGroup {
                items: ModelRc::new(VecModel::from(rows)),
            }
        })
        .collect()
}

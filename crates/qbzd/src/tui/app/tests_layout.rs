// crates/qbzd/src/tui/app/tests_layout.rs — the 80x24 floor + 120x30 wide
// render-snapshot tests for every section.

use crate::tui::widgets;

use super::messages::{Screen, SCREENS};
use super::nav::Focus;
use super::tests_support::{bare_app, render};

#[test]
fn every_section_fits_the_80x24_floor() {
    for screen in SCREENS {
        for focus in [Focus::Nav, Focus::Content] {
            let app = bare_app(screen, focus);
            let out = render(&app, 80, 24);
            // The shell rendered (not the too-small guard).
            assert!(
                out.contains("Qoqobuz Daemon Setup"),
                "header missing for {screen:?}/{focus:?}"
            );
            assert!(
                !out.contains("terminal too small"),
                "80x24 must not trip the resize guard for {screen:?}"
            );
            // The sidebar and version chrome are present.
            assert!(out.contains("Account"), "sidebar missing for {screen:?}");
            let version = format!("qbzd {}", env!("CARGO_PKG_VERSION"));
            assert!(out.contains(&version), "version chrome missing for {screen:?}");
        }
    }
}

#[test]
fn resize_guard_still_fires_below_the_floor() {
    let app = bare_app(Screen::Audio, Focus::Content);
    let out = render(&app, 79, 24);
    assert!(out.contains("terminal too small"));
}

// ---- 120×30 wide: the roomy sidebar tier + every section still renders (FB5) ----

#[test]
fn every_section_fits_the_120x30_wide() {
    for screen in SCREENS {
        for focus in [Focus::Nav, Focus::Content] {
            let app = bare_app(screen, focus);
            let out = render(&app, 120, 30);
            assert!(out.contains("Qoqobuz Daemon Setup"), "header missing {screen:?}/{focus:?}");
            assert!(
                !out.contains("terminal too small"),
                "120x30 must not trip the resize guard for {screen:?}"
            );
            // Wide tier: labels spell out and each name carries a dim summary.
            assert!(out.contains("Import / Export"), "wide sidebar label missing {screen:?}");
            assert!(
                out.contains("output · bit-perfect"),
                "wide sidebar summary missing {screen:?}"
            );
        }
    }
}

#[test]
fn sidebar_is_compact_at_the_floor_and_wide_above_100() {
    // At the 80-col floor the compact label is used, not the spelled-out one.
    let floor = render(&bare_app(Screen::Audio, Focus::Nav), 80, 24);
    assert!(floor.contains("Import/Exp"), "compact label at the floor");
    assert!(!floor.contains("Import / Export"), "no wide label at the floor");
    // The 28-col sidebar is at least double the 14-col compact one.
    assert!(widgets::sidebar_width(120) >= 2 * widgets::sidebar_width(80));
}

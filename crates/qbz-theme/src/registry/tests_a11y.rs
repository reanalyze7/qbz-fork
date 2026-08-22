//! P3 — Accessibility contrast unit tests (the plan's "WCAG/APCA unit tests").
//!
//! Every threshold here is the documented target from 99-MIGRATION-PLAN.md
//! Part B. If an assertion fails, the HEX is wrong vs Part B — fix the value to
//! match the verified palette, NEVER weaken the test.
//!
//! Split across several files for the 130-line budget:
//!   - this file: wcag-light/dark + high-contrast (both polarities)
//!   - `tests_a11y_colorblind.rs`: colorblind hue-separation + CVD simulation
//!   - `tests_a11y_global.rs`: the "every registered theme" completeness check

pub(crate) mod helpers;

use super::*;
use crate::color::{apca_lc, contrast_ratio};
use helpers::{AAA_BODY, NON_TEXT};

// ---- wcag-light: body text AAA, accent AAA, status AAA ----------------
#[test]
fn wcag_light_meets_aaa() {
    let c = wcag_light();
    // text-primary on bg-primary >= 7.0:1 (AAA) — Part B: 17.40:1 / Lc 104.3
    assert!(
        contrast_ratio(c.text_primary, c.surface_main) >= AAA_BODY,
        "wcag-light text-primary {:.2}",
        contrast_ratio(c.text_primary, c.surface_main)
    );
    // text-muted on bg-primary >= 7.0:1 (AAA, exactly) — Part B 7.00:1
    assert!(contrast_ratio(c.text_muted, c.surface_main) >= AAA_BODY);
    // accent + btn-text on accent >= 7.0:1 — Part B 7.98:1
    assert!(contrast_ratio(c.accent, c.surface_main) >= AAA_BODY);
    assert!(contrast_ratio(c.accent_text, c.accent) >= AAA_BODY);
    // danger/warning/success text on bg-primary >= AAA
    assert!(contrast_ratio(c.danger, c.surface_main) >= AAA_BODY);
    assert!(contrast_ratio(c.warning, c.surface_main) >= AAA_BODY);
    assert!(contrast_ratio(c.success, c.surface_main) >= AAA_BODY);
    // non-text: border-strong + focus-ring >= 3:1
    assert!(contrast_ratio(c.border_strong, c.surface_main) >= NON_TEXT);
    assert!(contrast_ratio(c.focus_ring, c.surface_main) >= NON_TEXT);
    // APCA body gate (|Lc| >= 75) for primary text
    assert!(apca_lc(c.text_primary, c.surface_main).abs() >= 75.0);
}

// ---- wcag-dark: AAA + APCA, status on opaque tints --------------------
#[test]
fn wcag_dark_meets_aaa() {
    let c = wcag_dark();
    assert!(contrast_ratio(c.text_primary, c.surface_main) >= AAA_BODY); // 16.02:1
    assert!(contrast_ratio(c.text_secondary, c.surface_main) >= AAA_BODY);
    // text-muted is APCA-content by design but still clears AAA ratio (10.32:1)
    assert!(contrast_ratio(c.text_muted, c.surface_main) >= AAA_BODY);
    assert!(contrast_ratio(c.accent, c.surface_main) >= AAA_BODY); // 10.39:1
    // danger/warning/success text on their OPAQUE tint bg >= AAA
    assert!(contrast_ratio(c.danger, c.danger_bg) >= AAA_BODY); // 8.63:1
    assert!(contrast_ratio(c.warning, c.surface_main) >= AAA_BODY); // 12.53:1
    assert!(contrast_ratio(c.success, c.success_bg) >= AAA_BODY);
    // border-strong >= 3:1 on every surface tier (Part B: 4.11/3.76/3.31)
    for bg in [c.surface_main, c.surface_card, c.surface_elevated] {
        assert!(
            contrast_ratio(c.border_strong, bg) >= NON_TEXT,
            "wcag-dark border-strong {:.2}",
            contrast_ratio(c.border_strong, bg)
        );
    }
    assert!(contrast_ratio(c.focus_ring, c.surface_main) >= NON_TEXT);
    assert!(apca_lc(c.text_primary, c.surface_main).abs() >= 75.0);
}

// ---- High Contrast (both polarities): >= the wcag bar + interactive ---
#[test]
fn high_contrast_dark_beats_wcag_dark() {
    let hc = high_contrast();
    let wd = wcag_dark();
    let hc_tp = contrast_ratio(hc.text_primary, hc.surface_main);
    let wd_tp = contrast_ratio(wd.text_primary, wd.surface_main);
    // HC must be at least as high-contrast as wcag-dark (no regression).
    assert!(
        hc_tp >= wd_tp,
        "HC-dark text/bg {:.2} should be >= wcag-dark {:.2}",
        hc_tp,
        wd_tp
    );
    assert!(hc_tp >= 19.0); // Part B: 19.80:1
    // reciprocal cyan: accent as text AND as a fill under btn-text
    assert!(contrast_ratio(hc.accent, hc.surface_main) >= AAA_BODY); // 11.67:1
    assert!(contrast_ratio(hc.accent_text, hc.accent) >= AAA_BODY); // 12.38:1
    // interactive non-text tokens >= 3:1
    assert!(contrast_ratio(hc.border_strong, hc.surface_main) >= NON_TEXT);
    assert!(contrast_ratio(hc.focus_ring, hc.surface_main) >= NON_TEXT); // 13.76:1
    assert!(contrast_ratio(hc.border_subtle, hc.surface_main) >= NON_TEXT); // 4.61:1
}

#[test]
fn high_contrast_light_beats_wcag_light() {
    let hc = high_contrast_light();
    let wl = wcag_light();
    let hc_tp = contrast_ratio(hc.text_primary, hc.surface_main);
    let wl_tp = contrast_ratio(wl.text_primary, wl.surface_main);
    assert!(
        hc_tp >= wl_tp,
        "HC-light text/bg {:.2} should be >= wcag-light {:.2}",
        hc_tp,
        wl_tp
    );
    assert!(hc_tp >= 20.0); // Part B: 21.00:1
    // reciprocal deep blue: accent as text AND btn-text under accent fill
    assert!(contrast_ratio(hc.accent, hc.surface_main) >= AAA_BODY); // 11.22:1
    assert!(contrast_ratio(hc.accent_text, hc.accent) >= AAA_BODY);
    // corrected warning #5e4b00 must clear AAA on white (8.46:1)
    assert!(
        contrast_ratio(hc.warning, hc.surface_main) >= AAA_BODY,
        "HC-light warning {:.2} (corrected #5e4b00 should be 8.46:1)",
        contrast_ratio(hc.warning, hc.surface_main)
    );
    assert_eq!(hc.warning, Rgba::rgb(0x5e, 0x4b, 0x00)); // the applied correction
    assert!(contrast_ratio(hc.border_strong, hc.surface_main) >= NON_TEXT);
    assert!(contrast_ratio(hc.focus_ring, hc.surface_main) >= NON_TEXT);
    assert!(contrast_ratio(hc.border_subtle, hc.surface_main) >= NON_TEXT); // 5.74:1
}


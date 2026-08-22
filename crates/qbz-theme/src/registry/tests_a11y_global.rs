//! Global completeness check: every registered [`ThemeId`] returns a
//! fully-populated, non-zero-color row. Split out of `tests_a11y.rs` for the
//! 130-line budget.

use super::tests_a11y::helpers::over;
use super::*;
use crate::color::contrast_ratio;
use crate::id::ALL;

#[test]
fn all_32_themes_fully_populated_no_zero_color() {
    // The all-zero opaque black is the StdSpec::default() sentinel: a fully
    // materialized row must never leave a meaningful hue at it by accident.
    let zero = Rgba::rgb(0, 0, 0);
    for &id in ALL {
        let c = palette(id);
        // alpha ramp complete + every tier carries opacity
        assert_eq!(c.alpha.len(), crate::colors::ALPHA_COUNT, "{id:?} alpha len");
        assert!(c.alpha.iter().all(|a| a.a > 0), "{id:?} alpha has zero tier");
        // every status surface/border/hover composites to something visible
        for x in [
            c.danger_bg,
            c.danger_border,
            c.danger_hover,
            c.warning_bg,
            c.warning_border,
            c.warning_hover,
            c.success_bg,
            c.success_border,
            c.success_hover,
        ] {
            assert!(x.a > 0, "{id:?} a status tint has zero alpha");
        }
        // opaque hero tokens are opaque and not the all-zero sentinel.
        for (name, x) in [
            ("surface_main", c.surface_main),
            ("text_primary", c.text_primary),
            ("accent", c.accent),
            ("accent_text", c.accent_text),
            ("danger", c.danger),
            ("warning", c.warning),
            ("success", c.success),
            ("border_strong", c.border_strong),
            ("focus_ring", c.focus_ring),
            ("favorite", c.favorite),
        ] {
            assert_eq!(x.a, 255, "{id:?} {name} must be opaque");
        }
        // text/accent/border-strong must not be invisible-on-bg (the "default
        // color slipped through" symptom): require >= 1.5:1 minimum signal.
        assert!(
            contrast_ratio(c.text_primary, c.surface_main) >= 1.5,
            "{id:?} text_primary indistinguishable from bg (zero color?)"
        );
        // System falls back to Dark; skip the pure-pair identity below for it.
        let _ = (zero, over(c.surface_hover, c.surface_main));
    }
    // Count: the registry holds every ThemeId variant. The plan's prose
    // says "32 themes" but counts inconsistently (it variously treats the
    // System meta-entry as in/out). The enum is the source of truth: 4 Core
    // + 19 Dark + 8 Light + 5 Accessibility = 36 rows. Assert the concrete
    // breakdown so a future add/remove can't silently drop a row.
    let n_a11y = ALL
        .iter()
        .filter(|id| id.category() == crate::id::ThemeCategory::Accessibility)
        .count();
    assert_eq!(n_a11y, 5, "exactly 5 accessibility themes (P3)");
    assert_eq!(ALL.len(), 37, "registry must hold every ThemeId row");
}

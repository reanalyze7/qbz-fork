//! `colorblind` accessibility theme: text contrast + status hue distinctness
//! under simulated red-green color vision deficiency. Split out of
//! `tests_a11y.rs` for the 130-line budget; shares helpers with it via
//! `tests_a11y::helpers`.

use super::tests_a11y::helpers::{delta_e, simulate_deutan, AA_NORMAL, AAA_BODY, NON_TEXT};
use super::*;
use crate::color::contrast_ratio;

#[test]
fn colorblind_text_passes_aa() {
    let c = colorblind();
    // text-primary AAA; muted AA on the lightest tier (Part B 5.88:1)
    assert!(contrast_ratio(c.text_primary, c.surface_main) >= AAA_BODY);
    assert!(contrast_ratio(c.text_muted, c.surface_elevated) >= AA_NORMAL);
    // accent/danger/warning double as text in ~300 places: AA on all tiers
    for bg in [c.surface_main, c.surface_card, c.surface_elevated] {
        assert!(contrast_ratio(c.accent, bg) >= AA_NORMAL, "accent on {bg:?}");
        assert!(contrast_ratio(c.danger, bg) >= AA_NORMAL, "danger on {bg:?}");
        assert!(contrast_ratio(c.warning, bg) >= AA_NORMAL, "warning on {bg:?}");
    }
    // success: AA-normal on primary/secondary (Part B/C.6); body text on the
    // lightest tier routes to success-hover (must clear AA-normal there).
    assert!(contrast_ratio(c.success, c.surface_main) >= AA_NORMAL); // 4.99:1
    assert!(contrast_ratio(c.success, c.surface_card) >= AA_NORMAL); // 4.52:1
    assert!(contrast_ratio(c.success_hover, c.surface_elevated) >= AA_NORMAL); // 6.04:1
    // focus-ring high-tone blue >= 3:1 non-text (Part B 8.53:1)
    assert!(contrast_ratio(c.focus_ring, c.surface_main) >= NON_TEXT);
}

#[test]
fn colorblind_status_hues_stay_distinct_under_cvd() {
    let c = colorblind();
    // The decisive separation (Part B): danger vs warning under red-green
    // CVD must stay clearly distinct (delete vs caution must not collapse).
    // Part B reports ΔE 34.81 under deuteranopia; assert a strong margin.
    let d_sim = simulate_deutan(c.danger);
    let w_sim = simulate_deutan(c.warning);
    let dw = delta_e(d_sim, w_sim);
    assert!(
        dw >= 15.0,
        "colorblind danger vs warning under deutan ΔE {dw:.2} should stay distinct (Part B 34.81)"
    );
    // accent vs danger also separable under red-green CVD (Part B protan 10.40).
    let a_sim = simulate_deutan(c.accent);
    let ad = delta_e(a_sim, d_sim);
    assert!(
        ad >= 8.0,
        "colorblind accent vs danger under deutan ΔE {ad:.2} should stay distinct"
    );
    // accent vs warning likewise (blue vs amber, the easy axis).
    let aw = delta_e(a_sim, w_sim);
    assert!(aw >= 15.0, "colorblind accent vs warning under deutan ΔE {aw:.2}");
    // sanity: the unsimulated hues are obviously distinct too.
    assert!(delta_e(c.danger, c.warning) >= 20.0);
}

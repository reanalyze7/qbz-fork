// crates/qbzd/src/tui/screens/wizard/tests_step.rs — step-transition table,
// Esc-abandon gating, and breadcrumb tests.

use super::*;

#[test]
fn step_transition_table_is_linear_and_bounded() {
    assert_eq!(next_step(WStep::Welcome), Some(WStep::Check));
    assert_eq!(next_step(WStep::Check), Some(WStep::SelectDacs));
    assert_eq!(next_step(WStep::SelectDacs), Some(WStep::Review));
    assert_eq!(next_step(WStep::Review), Some(WStep::Test));
    assert_eq!(next_step(WStep::Test), Some(WStep::Done));
    assert_eq!(next_step(WStep::Done), None); // terminal
    assert_eq!(prev_step(WStep::Welcome), None); // initial
    assert_eq!(prev_step(WStep::Done), Some(WStep::Test));
    // Round trip through every step.
    for s in STEP_ORDER {
        if let Some(n) = next_step(s) {
            assert_eq!(prev_step(n), Some(s));
        }
    }
}

#[test]
fn esc_confirms_abandon_only_on_middle_steps() {
    let mut w = WizardState::new();
    assert!(matches!(w.on_escape(), crate::tui::app::ScreenAction::Back)); // Welcome
    w.step = WStep::Review;
    assert!(matches!(w.on_escape(), crate::tui::app::ScreenAction::WizardAbandon));
    w.step = WStep::Done;
    assert!(matches!(w.on_escape(), crate::tui::app::ScreenAction::Back));
}

#[test]
fn only_welcome_declines_to_claim_horizontal() {
    // Welcome has nothing that consumes ← (the CTA only listens for
    // Enter/Space), so it must NOT claim ←/→ — otherwise ← is silently
    // swallowed by a no-op retreat() instead of dropping focus to the
    // sidebar like every other section.
    let mut w = WizardState::new();
    for step in STEP_ORDER {
        w.step = step;
        assert_eq!(
            w.claims_horizontal(),
            step != WStep::Welcome,
            "step {step:?} claims_horizontal mismatch"
        );
    }
}

#[test]
fn breadcrumb_reflects_the_current_step() {
    let mut w = WizardState::new();
    assert_eq!(w.editing_label(), Some(crate::tui::strings::WIZ_STEP_WELCOME));
    w.step = WStep::Review;
    assert_eq!(w.editing_label(), Some(crate::tui::strings::WIZ_STEP_REVIEW));
}

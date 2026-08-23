// crates/qbzd/src/tui/screens/wizard/tests_select.rs — Select-DACs gating,
// candidate/manual precedence, and Review-scroll clamping tests.

use super::*;
use crate::tui::app::ScreenAction;
use crate::tui::wizard_core::DacCandidateData;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn select_gate_blocks_advance_without_a_selection() {
    let mut w = WizardState::new();
    w.step = WStep::SelectDacs;
    w.detected = true;
    // No candidate checked, no manual → advance is refused, step unchanged.
    let action = w.advance();
    assert!(matches!(action, ScreenAction::Consumed));
    assert_eq!(w.step, WStep::SelectDacs);
    assert!(w.gate_note.is_some());

    // A manual node satisfies the gate.
    w.manual_node = Some("alsa_output.usb-x.analog-stereo".to_string());
    let action = w.advance();
    assert!(matches!(action, ScreenAction::WizardGenConfigs(_)));
    assert_eq!(w.step, WStep::Review);
}

#[test]
fn checked_dacs_prefers_candidates_then_manual() {
    let mut w = WizardState::new();
    w.set_candidates(vec![
        DacCandidateData {
            id: "node-a".into(),
            description: "DAC A".into(),
            bus: "usb".into(),
            is_default: false,
            looks_like_dac: true,
            rates_label: "44.1 / 192 kHz".into(),
        },
        DacCandidateData {
            id: "node-b".into(),
            description: "Monitor".into(),
            bus: "".into(),
            is_default: false,
            looks_like_dac: false,
            rates_label: "".into(),
        },
    ]);
    // Only the likely DAC is pre-checked.
    assert_eq!(w.checked_dacs(), vec![("node-a".to_string(), "DAC A".to_string())]);
    // With nothing checked, the manual node is the fallback.
    for c in &mut w.candidates {
        c.checked = false;
    }
    w.manual_node = Some("alsa_output.usb-y".to_string());
    assert_eq!(
        w.checked_dacs(),
        vec![("alsa_output.usb-y".to_string(), "alsa_output.usb-y".to_string())]
    );
}

#[test]
fn review_scroll_page_down_never_overscrolls_past_the_last_block() {
    let mut w = super::tests_render::populated(); // one config block
    w.step = WStep::Review;
    let max = w.max_review_scroll();
    assert!(max > 0, "the populated fixture should have real content to clamp against");

    let page_down = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
    // Mash PageDown well past the content height.
    for _ in 0..20 {
        w.handle_key(page_down);
    }
    assert_eq!(w.review_scroll, max);
}

// crates/qbzd/src/tui/screens/wizard/tests_render.rs — 80x24 / 120x30
// render-snapshot tests for every wizard step, plus the shared `populated`
// fixture (also used by `tests_select.rs`'s scroll-clamp test).

use super::*;
use crate::tui::app::DrawCtx;
use crate::tui::wizard_core::{DacCandidateData, DacConfigData};
use qbz_audio::NegotiatedRate;

pub(super) fn populated() -> WizardState {
    let mut w = WizardState::new();
    w.set_health(qbz_audio::AudioStackHealth {
        wireplumber_active: true,
        has_pw_dump: true,
        cpal_sees_pipewire: true,
        has_pactl: true,
        any_devices: true,
    });
    w.set_candidates(vec![DacCandidateData {
        id: "alsa_output.usb-Cambridge-00.analog-stereo".into(),
        description: "Cambridge DacMagic".into(),
        bus: "usb".into(),
        is_default: true,
        looks_like_dac: true,
        rates_label: "44.1 / 96 / 192 kHz".into(),
    }]);
    w.set_configs(vec![DacConfigData {
        name: "Cambridge DacMagic".into(),
        node_name: "alsa_output.usb-Cambridge-00.analog-stereo".into(),
        pipewire_conf: "context.properties = { default.clock.allowed-rates = [ 44100 192000 ] }".into(),
        pulse_conf: "stream.rules = [ ... ]".into(),
        wireplumber_conf: "monitor.alsa.rules = [ ... ]".into(),
    }]);
    w.set_test_result(Some((192000, 24)), Some(NegotiatedRate {
        sample_rate: 192000,
        format: "S32_LE".into(),
        channels: 2,
    }), None);
    w
}

fn render_step(w: &WizardState) -> String {
    render_step_sized(w, 80, 24)
}

fn render_step_sized(w: &WizardState, width: u16, height: u16) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let ctx = DrawCtx { status: None };
    terminal
        .draw(|f| {
            let area = f.area();
            w.draw(f, area, &ctx);
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut out = String::new();
    for y in 0..height {
        for x in 0..width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn every_wizard_step_renders_on_the_80x24_floor() {
    let mut w = populated();
    let expect: [(WStep, &str); 6] = [
        (WStep::Welcome, "HiFi"),
        (WStep::Check, "ready"),
        (WStep::SelectDacs, "Cambridge"),
        (WStep::Review, "Cambridge"),
        (WStep::Test, "DAC:"),
        (WStep::Done, "All set"),
    ];
    for (step, needle) in expect {
        w.step = step;
        let out = render_step(&w);
        assert!(out.contains(needle), "step {step:?} should render {needle:?}");
    }
    // The Review step exposes the copy affordance + the never-writes footer.
    w.step = WStep::Review;
    let review = render_step(&w);
    assert!(review.contains("→ ~/.config"), "review shows the target file paths");
    assert!(review.contains("NEVER"), "review footer states the wizard never writes files");
}

#[test]
fn every_wizard_step_renders_at_120x30_wide() {
    // The wizard body draws into the content frame (no sidebar of its own);
    // this mirrors it at a comfortable wide size and asserts each step's
    // content survives the word-wrap rework without a panic (FB5).
    let mut w = populated();
    let expect: [(WStep, &str); 6] = [
        (WStep::Welcome, "HiFi"),
        (WStep::Check, "ready"),
        (WStep::SelectDacs, "Cambridge"),
        (WStep::Review, "Cambridge"),
        (WStep::Test, "DAC:"),
        (WStep::Done, "All set"),
    ];
    for (step, needle) in expect {
        w.step = step;
        let out = render_step_sized(&w, 120, 30);
        assert!(out.contains(needle), "wide step {step:?} should render {needle:?}");
    }
}

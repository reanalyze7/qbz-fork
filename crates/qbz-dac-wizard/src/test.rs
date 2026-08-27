//! Slice 9: self-service playback test (N6 read-back).

use std::sync::Mutex;

use slint::ComponentHandle;

use qbz_ui::{AppWindow, DacWizardState};

/// One curated test track (owner-provided). Resolved by id-hint first, then by
/// "artist title" search if the id 404s (a pulled license) — never raw-id-only.
pub struct TestSeed {
    pub depth: u32,
    pub rate: f64,
    pub id_hint: u64,
    pub artist: &'static str,
    pub title: &'static str,
}

pub const TEST_SEEDS: [TestSeed; 4] = [
    TestSeed { depth: 16, rate: 44100.0, id_hint: 19301386, artist: "George Harrison", title: "My Sweet Lord" },
    TestSeed { depth: 24, rate: 44100.0, id_hint: 266725027, artist: "Billie Eilish", title: "LUNCH" },
    TestSeed { depth: 24, rate: 96000.0, id_hint: 126886854, artist: "Iron Maiden", title: "Stratego" },
    TestSeed { depth: 24, rate: 192000.0, id_hint: 52265, artist: "Toto", title: "Africa" },
];

/// True if a resolved track matches this seed's family (rate + bit depth — the
/// two 44.1 seeds only differ by depth).
pub fn track_matches_seed(track: &qbz_models::Track, seed: &TestSeed) -> bool {
    let rate_ok = track
        .maximum_sampling_rate
        .map(|r| (r * 1000.0 - seed.rate).abs() < 1.0 || (r - seed.rate).abs() < 1.0)
        .unwrap_or(false);
    let depth_ok = track.maximum_bit_depth.map(|d| d == seed.depth).unwrap_or(false);
    rate_ok && depth_ok
}

/// Resolved test tracks, kept so the user can jump straight to any of them
/// (re-set the queue at the chosen index via the working play path).
static TEST_TRACKS: Mutex<Vec<qbz_models::Track>> = Mutex::new(Vec::new());

pub fn stash_test_tracks(tracks: Vec<qbz_models::Track>) {
    *TEST_TRACKS.lock().unwrap() = tracks;
}

pub fn test_tracks() -> Vec<qbz_models::Track> {
    TEST_TRACKS.lock().unwrap().clone()
}

/// Start the test: show the "playing" state. The read-back probes whichever
/// DAC is actively playing (scan), so no node needs to be stashed.
pub fn begin_test(window: &AppWindow) {
    let st = window.global::<DacWizardState>();
    st.set_test_playing(true);
    st.set_test_rate_matched(false);
    st.set_test_requested_label(qbz_i18n::t("Starting…").into());
    st.set_test_negotiated_label("".into());
}

pub fn end_test(window: &AppWindow) {
    window.global::<DacWizardState>().set_test_playing(false);
}

/// Guardrail: "Use my current queue" with an empty queue — show a hint instead
/// of starting a read-back that would just sit on "Nothing playing".
pub fn queue_empty_notice(window: &AppWindow) {
    let st = window.global::<DacWizardState>();
    st.set_test_playing(false);
    st.set_test_rate_matched(false);
    st.set_test_negotiated_label("".into());
    st.set_test_requested_label(
        qbz_i18n::t("Your queue is empty — add some tracks first, or press Play test.").into(),
    );
}

/// Apply one poll: the rate QBZ requested vs the DAC's real negotiated rate (N6).
pub fn apply_poll(
    window: &AppWindow,
    requested_rate: u32,
    requested_bits: u32,
    negotiated: Option<qbz_audio::NegotiatedRate>,
) {
    let st = window.global::<DacWizardState>();
    st.set_test_requested_label(if requested_rate > 0 {
        qbz_i18n::t_args(
            "Qoqobuz requesting {} · {}-bit",
            &[&khz(requested_rate), &requested_bits.to_string()],
        )
        .into()
    } else {
        qbz_i18n::t("Nothing playing").into()
    });
    match negotiated {
        Some(n) => {
            // The DAC's REAL hardware params (N6): rate + ALSA container format
            // (e.g. S32_LE = 24-bit in a 32-bit frame) + channels. This is the
            // bit-perfect proof — exactly what the hardware is clocked at.
            st.set_test_negotiated_label(
                qbz_i18n::t_args(
                    "DAC: {} · {} · {} ch",
                    &[&khz(n.sample_rate), &n.format, &n.channels.to_string()],
                )
                .into(),
            );
            // Truth signal: the DAC's real clock matches what QBZ asked for.
            st.set_test_rate_matched(requested_rate > 0 && n.sample_rate == requested_rate);
        }
        None => {
            st.set_test_negotiated_label(qbz_i18n::t("Waiting for the DAC to start playing…").into());
            st.set_test_rate_matched(false);
        }
    }
}

fn khz(hz: u32) -> String {
    if hz % 1000 == 0 {
        format!("{} kHz", hz / 1000)
    } else {
        format!("{:.1} kHz", hz as f64 / 1000.0)
    }
}

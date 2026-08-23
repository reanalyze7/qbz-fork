// ── self-service playback test (N6 read-back) ───────────────────────────────
use qbz_audio::NegotiatedRate;

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

/// The curated reference seed a live `(rate_hz, depth)` pair corresponds to, if
/// any. Non-test call site for [`track_matches_seed`]: the daemon test step
/// reads a hardware rate back (`negotiated_active_rate`) rather than resolving a
/// seed track through search, so this labels the played rate as a known
/// bit-perfect reference when it lines up.
pub fn seed_for_rate_depth(rate_hz: u32, depth: u32) -> Option<&'static TestSeed> {
    let track = qbz_models::Track {
        maximum_sampling_rate: Some(rate_hz as f64),
        maximum_bit_depth: Some(depth),
        ..Default::default()
    };
    TEST_SEEDS.iter().find(|s| track_matches_seed(&track, s))
}

/// "192 kHz" / "44.1 kHz" from Hz (shared by the test read-back rendering).
pub fn khz(hz: u32) -> String {
    if hz % 1000 == 0 {
        format!("{} kHz", hz / 1000)
    } else {
        format!("{:.1} kHz", hz as f64 / 1000.0)
    }
}

/// The DAC read-back line: real hardware rate · ALSA container format · channels
/// (N6). `S32_LE` = 24-bit carried in a 32-bit frame — this is the container, so
/// the wizard's "matched" verdict keys on the RATE, not the format string.
pub fn negotiated_label(n: &NegotiatedRate) -> String {
    format!("DAC: {} · {} · {} ch", khz(n.sample_rate), n.format, n.channels)
}

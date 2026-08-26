use super::loudness_meter::LoudnessMeter;

/// Sinus a -20 dBFS : une source deterministe dont la loudness est connue
/// a peu pres, ce qui suffit a verifier que le meter mesure vraiment.
fn sine(rate: u32, secs: f32, amp: f32) -> Vec<f32> {
    let n = (rate as f32 * secs) as usize;
    (0..n)
        .flat_map(|i| {
            let v = amp * (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / rate as f32).sin();
            [v, v]
        })
        .collect()
}

#[test]
fn mesure_un_signal_reel() {
    let mut m = LoudnessMeter::new(48000, 2).unwrap();
    assert!(m.feed(&sine(48000, 5.0, 0.1)));
    let lufs = m.integrated_lufs().expect("mesure exploitable");
    assert!((-26.0..-14.0).contains(&lufs), "lufs = {}", lufs);
    assert!((m.peak() - 0.1).abs() < 0.01);
    assert_eq!(m.frames_fed(), 240_000);
}

#[test]
fn le_silence_n_est_pas_une_mesure() {
    let mut m = LoudnessMeter::new(48000, 2).unwrap();
    m.feed(&vec![0.0; 48000 * 2 * 5]);
    assert!(m.integrated_lufs().is_none());
}

#[test]
fn court_terme_disponible_avant_l_integree() {
    let mut m = LoudnessMeter::new(48000, 2).unwrap();
    m.feed(&sine(48000, 1.5, 0.1));
    assert!(m.shortterm_lufs().is_some());
}

#[test]
fn reset_repart_de_zero() {
    let mut m = LoudnessMeter::new(48000, 2).unwrap();
    m.feed(&sine(48000, 2.0, 0.1));
    m.reset(44100, 2);
    assert_eq!(m.frames_fed(), 0);
    assert_eq!(m.peak(), 0.0);
}

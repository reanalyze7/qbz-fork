use super::*;

#[test]
fn test_db_to_linear() {
    // 0 dB = factor 1.0
    assert!((db_to_linear(0.0) - 1.0).abs() < 0.001);
    // -6 dB ≈ 0.501
    assert!((db_to_linear(-6.0) - 0.501).abs() < 0.01);
    // +6 dB ≈ 1.995
    assert!((db_to_linear(6.0) - 1.995).abs() < 0.01);
    // -20 dB = 0.1
    assert!((db_to_linear(-20.0) - 0.1).abs() < 0.001);
}

#[test]
fn test_calculate_gain_factor_at_reference() {
    // At -18 LUFS target (ReplayGain reference), gain_db should pass through directly
    let rg = ReplayGainData {
        gain_db: -3.0,
        peak: Some(0.9),
    };
    let factor = calculate_gain_factor(&rg, -18.0);
    // -3 dB → ~0.708
    assert!((factor - 0.708).abs() < 0.01);
}

#[test]
fn test_calculate_gain_factor_with_target_adjustment() {
    // At -14 LUFS target, we add +4 dB to the RG gain
    let rg = ReplayGainData {
        gain_db: -3.0,
        peak: Some(0.5),
    };
    let factor = calculate_gain_factor(&rg, -14.0);
    // -3 + 4 = +1 dB → ~1.122
    assert!((factor - 1.122).abs() < 0.01);
}

#[test]
fn test_clipping_prevention_with_peak() {
    // High positive gain but peak close to 1.0 — should be capped
    let rg = ReplayGainData {
        gain_db: 10.0,
        peak: Some(0.95),
    };
    let factor = calculate_gain_factor(&rg, -18.0);
    // max_safe_gain = 1/0.95 ≈ 1.053, which is less than db_to_linear(10) ≈ 3.162
    assert!((factor - (1.0 / 0.95)).abs() < 0.01);
}

#[test]
fn test_clipping_prevention_without_peak() {
    // High gain without peak data — capped at +6 dB
    let rg = ReplayGainData {
        gain_db: 12.0,
        peak: None,
    };
    let factor = calculate_gain_factor(&rg, -18.0);
    assert!((factor - db_to_linear(6.0)).abs() < 0.01);
}

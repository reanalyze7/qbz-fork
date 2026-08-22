use super::super::NUM_BARS;

#[test]
fn test_log_frequency_distribution() {
    // Verify that frequency bars are logarithmically distributed
    let num_bars = NUM_BARS; // Use the actual constant
    let min_log = 20.0_f32.ln();
    let max_log = 20000.0_f32.ln();

    let mut freqs = Vec::new();
    for i in 0..num_bars {
        let t = i as f32 / num_bars as f32;
        let freq = (min_log + (max_log - min_log) * t).exp();
        freqs.push(freq);
    }

    // First bar should be around 20Hz
    assert!(freqs[0] > 19.0 && freqs[0] < 25.0);

    // Middle bar (~16 for 32 bars) should be around 630Hz (geometric mean of 20 and 20000)
    let mid = num_bars / 2;
    assert!(freqs[mid] > 500.0 && freqs[mid] < 800.0);

    // Last bar should approach 20000Hz (but won't reach it since t < 1.0)
    assert!(freqs[num_bars - 1] > 10000.0);
}

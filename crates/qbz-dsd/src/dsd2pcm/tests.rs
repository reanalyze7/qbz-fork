use super::*;

#[test]
fn silence_in_near_zero_out() {
    let mut d = Dsd2Pcm::new();
    let mut out = Vec::new();
    // 0x69 = 01101001: the classic DSD silence pattern (DC-balanced).
    d.translate(&[0x69; 4096], false, &mut out);
    assert_eq!(out.len(), 4096);
    // Skip the filter warm-up, then expect (near) silence.
    for &s in &out[64..] {
        assert!(s.abs() < 1e-3, "sample {s} not near zero");
    }
}

#[test]
fn all_ones_approaches_full_scale_positive() {
    let mut d = Dsd2Pcm::new();
    let mut out = Vec::new();
    d.translate(&[0xFF; 4096], false, &mut out);
    let tail = &out[512..];
    let avg: f32 = tail.iter().sum::<f32>() / tail.len() as f32;
    // DC gain of the filter is ~1.0 for the all-ones (+1) stream.
    assert!(avg > 0.8, "avg {avg} too low for all-ones input");
}

#[test]
fn lsb_msb_orders_differ_only_by_bit_reversal() {
    let mut a = Dsd2Pcm::new();
    let mut b = Dsd2Pcm::new();
    let (mut oa, mut ob) = (Vec::new(), Vec::new());
    let pattern: Vec<u8> = (0..=255u8).cycle().take(2048).collect();
    let reversed: Vec<u8> = pattern.iter().map(|&x| tables().bitreverse[x as usize]).collect();
    a.translate(&pattern, false, &mut oa);
    b.translate(&reversed, true, &mut ob);
    assert_eq!(oa, ob);
}

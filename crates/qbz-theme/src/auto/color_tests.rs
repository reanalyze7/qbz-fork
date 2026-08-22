use super::*;

#[test]
fn luminance_black_white() {
    assert!((PaletteColor::new(0, 0, 0).luminance() - 0.0).abs() < 1e-4);
    assert!((PaletteColor::new(255, 255, 255).luminance() - 1.0).abs() < 1e-4);
}

#[test]
fn contrast_ratio_bw_is_21() {
    let ratio = PaletteColor::new(0, 0, 0).contrast_ratio(&PaletteColor::new(255, 255, 255));
    assert!((ratio - 21.0).abs() < 0.1);
}

#[test]
fn hsl_roundtrip() {
    let c = PaletteColor::new(66, 133, 244);
    let (h, s, l) = c.to_hsl();
    let back = PaletteColor::from_hsl(h, s, l);
    assert!((c.r as i16 - back.r as i16).unsigned_abs() <= 1);
    assert!((c.g as i16 - back.g as i16).unsigned_abs() <= 1);
    assert!((c.b as i16 - back.b as i16).unsigned_abs() <= 1);
}

#[test]
fn saturation_gray_is_zero() {
    assert!(PaletteColor::new(128, 128, 128).saturation() < 0.01);
}

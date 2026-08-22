use super::*;

#[test]
fn hex_roundtrip() {
    assert_eq!(Rgba::from_hex("#0f0f0f"), Some(Rgba::rgb(15, 15, 15)));
    assert_eq!(Rgba::from_hex("ffffff"), Some(Rgba::rgb(255, 255, 255)));
    assert_eq!(Rgba::from_hex("#ffffff14"), Some(Rgba::rgba(255, 255, 255, 0x14)));
    assert_eq!(Rgba::from_hex("nope"), None);
}

#[test]
fn black_white_contrast_is_21() {
    let r = contrast_ratio(Rgba::rgb(0, 0, 0), Rgba::rgb(255, 255, 255));
    assert!((r - 21.0).abs() < 0.01, "got {r}");
}

#[test]
fn contrast_is_symmetric_and_min_one() {
    let a = Rgba::rgb(0x42, 0x85, 0xf4);
    let b = Rgba::rgb(0x0f, 0x0f, 0x0f);
    assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 1e-9);
    assert!(contrast_ratio(a, a) >= 0.999);
}

#[test]
fn apca_sign_convention() {
    // light text on dark bg -> negative
    let lc = apca_lc(Rgba::rgb(255, 255, 255), Rgba::rgb(15, 15, 15));
    assert!(lc < -60.0, "white on near-black should be strongly negative, got {lc}");
    // dark text on light bg -> positive
    let lc = apca_lc(Rgba::rgb(0, 0, 0), Rgba::rgb(255, 255, 255));
    assert!(lc > 90.0, "black on white should be strongly positive, got {lc}");
}

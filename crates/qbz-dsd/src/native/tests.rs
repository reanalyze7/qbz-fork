use super::*;

#[test]
fn word_packing_endianness() {
    // Direct math checks (pack_word needs an instance; test the formulas).
    let b = [0xAAu8, 0xBB, 0xCC, 0xDD];
    assert_eq!(i32::from_le_bytes(b), 0xDDCCBBAAu32 as i32); // BE layout
    assert_eq!(i32::from_be_bytes(b), 0xAABBCCDDu32 as i32); // LE layout
    assert_eq!(NATIVE_DSD_SILENCE_U32, i32::from_le_bytes([0x69; 4]));
    assert_eq!(NATIVE_DSD_SILENCE_U32, i32::from_be_bytes([0x69; 4]));
}

#[test]
fn native_rate_math() {
    assert_eq!(native_u32_rate(2_822_400), 88_200);
    assert_eq!(native_u32_rate(11_289_600), 352_800);
}

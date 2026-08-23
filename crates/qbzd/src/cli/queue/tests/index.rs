use crate::cli::queue::{cli_index_to_api, cli_position};

// -------------------- index translation (both directions) --------------------

#[test]
fn cli_index_to_api_shifts_1based_to_0based() {
    // 02 §3.3.15's own example state: current_index=1 (0-based) IS
    // position 2 (1-based) in the §2.2 table — this is the inverse.
    assert_eq!(cli_index_to_api(1), Ok(0));
    assert_eq!(cli_index_to_api(2), Ok(1));
    assert_eq!(cli_index_to_api(14), Ok(13));
}

#[test]
fn cli_index_to_api_rejects_position_zero() {
    assert!(cli_index_to_api(0).is_err());
}

#[test]
fn cli_position_shifts_0based_to_1based() {
    assert_eq!(cli_position(0), 1);
    assert_eq!(cli_position(1), 2);
    assert_eq!(cli_position(13), 14);
}

#[test]
fn index_translation_round_trips() {
    for api_index in 0..20usize {
        assert_eq!(cli_index_to_api(cli_position(api_index)), Ok(api_index));
    }
}

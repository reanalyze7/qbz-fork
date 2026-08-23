use super::{ev, q};
use crate::playback_driver::{advance_state, next_playable, plan_tick, DriverAction, DriverState};

#[test]
fn skip_walk_bounds() {
    assert_eq!(next_playable(&[(2, false), (3, false), (4, true)], 50), Some((2, 4)));
    let all_bad: Vec<(u64, bool)> = (0..60).map(|i| (i, false)).collect();
    assert_eq!(next_playable(&all_bad, 50), None); // bounded — never walks forever
}

#[test]
fn position_save_cadence_11_ticks() {
    let mut s = DriverState::after(&ev(1, true, 10, 581));
    for tick in 1..=11u32 {
        let e = ev(1, true, 10 + tick as u64, 581);
        let a = plan_tick(&s, &e, &q(1, &[], "off", None), None);
        if tick == 11 {
            assert!(a.contains(&DriverAction::SavePosition(21)));
        } else {
            assert!(!a.iter().any(|x| matches!(x, DriverAction::SavePosition(_))));
        }
        s = advance_state(&s, &e, &a);
    }
}

#[test]
fn stream_error_latches() {
    let s = DriverState::after(&ev(1, true, 10, 581));
    let a = plan_tick(
        &s,
        &ev(1, false, 10, 581),
        &q(1, &[], "off", None),
        Some("ALSA device disappeared"),
    );
    assert!(a.contains(&DriverAction::LatchError("ALSA device disappeared".into())));
}

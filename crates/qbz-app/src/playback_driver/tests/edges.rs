use super::{ev, q};
use crate::playback_driver::{advance_state, plan_tick, DriverAction, DriverState};

#[test]
fn end_edge_advances() {
    let s = DriverState::after(&ev(1, true, 580, 581));
    let a = plan_tick(&s, &ev(1, false, 581, 581), &q(1, &[(2, true)], "off", None), None);
    assert!(a.contains(&DriverAction::AdvanceAndPlay));
    assert!(a.contains(&DriverAction::ReportEdge)); // play-state edge
}

#[test]
fn mid_track_pause_does_not_advance() {
    let s = DriverState::after(&ev(1, true, 100, 581));
    let a = plan_tick(&s, &ev(1, false, 100, 581), &q(1, &[(2, true)], "off", None), None);
    assert!(!a.contains(&DriverAction::AdvanceAndPlay));
    assert!(a.contains(&DriverAction::ReportEdge));
}

#[test]
fn stop_after_one_shot() {
    let s = DriverState::after(&ev(42, true, 580, 581));
    let a = plan_tick(
        &s,
        &ev(42, false, 581, 581),
        &q(42, &[(2, true)], "off", Some(42)),
        None,
    );
    assert!(a.contains(&DriverAction::PauseStopAfter));
    assert!(!a.contains(&DriverAction::AdvanceAndPlay));
}

#[test]
fn gapless_arms_exactly_once() {
    let mut e = ev(1, true, 300, 581);
    e.gapless_ready = true;
    e.gapless_next_track_id = 0;
    let s = DriverState::after(&ev(1, true, 299, 581));
    let queue = q(1, &[(2, true)], "off", None);
    let a1 = plan_tick(&s, &e, &queue, None);
    assert!(a1.contains(&DriverAction::ArmGapless(2)));
    let s2 = advance_state(&s, &e, &a1);
    let a2 = plan_tick(&s2, &e, &queue, None);
    assert!(!a2.iter().any(|x| matches!(x, DriverAction::ArmGapless(_))));
}

#[test]
fn repeat_one_advances_instead_of_finishing() {
    let s = DriverState::after(&ev(1, true, 580, 581));
    let a = plan_tick(&s, &ev(1, false, 581, 581), &q(1, &[], "one", None), None);
    assert!(a.contains(&DriverAction::AdvanceAndPlay));
    assert!(!a.contains(&DriverAction::QueueFinished));
}

#[test]
fn queue_finished_when_nothing_playable() {
    let s = DriverState::after(&ev(1, true, 580, 581));
    let a = plan_tick(&s, &ev(1, false, 581, 581), &q(1, &[(2, false)], "off", None), None);
    assert!(a.contains(&DriverAction::QueueFinished));
}

#[test]
fn seamless_gapless_transition_syncs_cursor() {
    let s = DriverState::after(&ev(1, true, 580, 581));
    let a = plan_tick(&s, &ev(2, true, 0, 547), &q(1, &[(2, true)], "off", None), None);
    assert!(a.contains(&DriverAction::SyncCursorTo(2)));
}

#[test]
fn duration_zero_never_advances() {
    let s = DriverState::after(&ev(1, true, 580, 581));
    let a = plan_tick(&s, &ev(1, false, 580, 0), &q(1, &[(2, true)], "off", None), None);
    assert!(!a.contains(&DriverAction::AdvanceAndPlay));
    assert!(a.contains(&DriverAction::ReportEdge)); // play-state edge
}

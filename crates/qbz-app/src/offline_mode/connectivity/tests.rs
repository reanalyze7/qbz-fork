use super::judge::ConnectivityJudge;
use super::route::{ipv4_has_default_route, ipv6_has_default_route};
use super::types::*;
use std::time::Instant;

fn now() -> Instant {
    Instant::now()
}

#[test]
fn first_success_goes_up_immediately() {
    let mut judge = ConnectivityJudge::new();
    assert_eq!(judge.on_probe(ProbeOutcome::Success, now()), JudgeAction::Idle);
    assert_eq!(judge.snapshot().state, Connectivity::Up);
}

#[test]
fn single_failure_while_up_does_not_flip_down() {
    let mut judge = ConnectivityJudge::new();
    judge.on_probe(ProbeOutcome::Success, now());

    let action = judge.on_probe(ProbeOutcome::Failure, now());
    assert!(matches!(action, JudgeAction::ConfirmAfter(_)));
    assert_eq!(
        judge.snapshot().state,
        Connectivity::Up,
        "must stay Up during the confirmation burst"
    );
}

#[test]
fn exhausted_confirmation_burst_flips_down() {
    let mut judge = ConnectivityJudge::new();
    judge.on_probe(ProbeOutcome::Success, now());

    // burst steps: CONFIRM_DELAYS.len() ConfirmAfter actions, then Down.
    let mut flips = 0;
    for _ in 0..(CONFIRM_DELAYS.len() + 1) {
        if judge.on_probe(ProbeOutcome::Failure, now()) == JudgeAction::Idle {
            flips += 1;
        }
    }
    assert_eq!(flips, 1);
    assert_eq!(judge.snapshot().state, Connectivity::Down);
}

#[test]
fn success_mid_burst_cancels_the_flip() {
    let mut judge = ConnectivityJudge::new();
    judge.on_probe(ProbeOutcome::Success, now());
    judge.on_probe(ProbeOutcome::Failure, now());
    judge.on_probe(ProbeOutcome::Success, now());
    assert_eq!(judge.snapshot().state, Connectivity::Up);

    // The next failure starts a FRESH burst (streak was reset).
    let action = judge.on_probe(ProbeOutcome::Failure, now());
    assert!(matches!(action, JudgeAction::ConfirmAfter(_)));
    assert_eq!(judge.snapshot().state, Connectivity::Up);
}

#[test]
fn down_recovers_on_single_success() {
    let mut judge = ConnectivityJudge::new();
    judge.on_probe(ProbeOutcome::Failure, now());
    assert_eq!(judge.snapshot().state, Connectivity::Down);

    judge.on_probe(ProbeOutcome::Success, now());
    assert_eq!(judge.snapshot().state, Connectivity::Up);
}

#[test]
fn captive_portal_is_down_with_hint() {
    let mut judge = ConnectivityJudge::new();
    judge.on_probe(ProbeOutcome::CaptivePortal, now());
    assert_eq!(judge.snapshot().state, Connectivity::Down);
    assert!(judge.snapshot().captive_portal);
}

#[test]
fn no_route_is_immediate_down() {
    let mut judge = ConnectivityJudge::new();
    judge.on_probe(ProbeOutcome::Success, now());
    judge.on_no_route();
    assert_eq!(judge.snapshot().state, Connectivity::Down);
}

#[test]
fn liveness_is_immediate_up_and_clears_streak() {
    let mut judge = ConnectivityJudge::new();
    judge.on_probe(ProbeOutcome::Success, now());
    judge.on_probe(ProbeOutcome::Failure, now()); // burst started
    judge.on_liveness();
    assert_eq!(judge.snapshot().state, Connectivity::Up);

    // Fresh burst required again.
    let action = judge.on_probe(ProbeOutcome::Failure, now());
    assert!(matches!(action, JudgeAction::ConfirmAfter(_)));
}

#[test]
fn ipv4_route_parse() {
    let with_default = "Iface\tDestination\tGateway\tFlags\n\
                        wlan0\t00000000\t0102A8C0\t0003\n\
                        wlan0\t0002A8C0\t00000000\t0001\n";
    let without_default = "Iface\tDestination\tGateway\tFlags\n\
                           wlan0\t0002A8C0\t00000000\t0001\n";
    let lo_only = "Iface\tDestination\tGateway\tFlags\n\
                   lo\t00000000\t00000000\t0001\n";
    assert!(ipv4_has_default_route(with_default));
    assert!(!ipv4_has_default_route(without_default));
    assert!(!ipv4_has_default_route(lo_only));
}

#[test]
fn ipv6_route_parse() {
    // dest(32) prefix(2) src(32) srcprefix(2) nexthop(32) metric refcnt use flags iface
    let with_default = "00000000000000000000000000000000 00 00000000000000000000000000000000 00 fe800000000000000000000000000001 00000400 00000000 00000000 00000003 wlan0\n";
    let non_default = "20010db8000000000000000000000000 40 00000000000000000000000000000000 00 00000000000000000000000000000000 00000400 00000000 00000000 00000001 wlan0\n";
    let lo_default = "00000000000000000000000000000000 00 00000000000000000000000000000000 00 00000000000000000000000000000000 00000400 00000000 00000000 00000003 lo\n";
    assert!(ipv6_has_default_route(with_default));
    assert!(!ipv6_has_default_route(non_default));
    assert!(!ipv6_has_default_route(lo_default));
}

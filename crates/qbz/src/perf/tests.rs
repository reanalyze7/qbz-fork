use std::time::{Duration, Instant};

use super::meter::FrameMeter;

fn meter() -> FrameMeter {
    FrameMeter::new(Duration::from_secs(1))
}

#[test]
fn no_sample_before_the_window_closes() {
    let mut m = meter();
    let t0 = Instant::now();
    for i in 0..30 {
        assert!(m.record(t0 + Duration::from_millis(i * 16)).is_none());
    }
}

#[test]
fn sixty_even_frames_read_as_sixty_fps() {
    let mut m = meter();
    let t0 = Instant::now();
    let mut got = None;
    // 61 frames: one opens the window, 60 intervals of 1/60 s close it.
    for i in 0..=60 {
        got = m.record(t0 + Duration::from_secs_f32(i as f32 / 60.0)).or(got);
    }
    let s = got.expect("the window must close on the frame that reaches 1 s");
    assert!((s.fps - 61.0).abs() < 1.0, "fps was {}", s.fps);
    assert!((s.frame_ms - 16.67).abs() < 0.1, "frame_ms was {}", s.frame_ms);
}

#[test]
fn worst_frame_is_the_longest_interval_not_the_average() {
    let mut m = meter();
    let t0 = Instant::now();
    // 10 quick frames, then one 250 ms stall, then the window closes.
    let mut t = 0.0_f32;
    for _ in 0..10 {
        t += 0.01;
        m.record(t0 + Duration::from_secs_f32(t));
    }
    t += 0.25;
    m.record(t0 + Duration::from_secs_f32(t));
    let s = m.record(t0 + Duration::from_secs_f32(1.05)).expect("window closes");
    assert!(s.worst_ms >= 249.0, "worst_ms was {} — the stall was flattened", s.worst_ms);
    assert!(s.frame_ms < s.worst_ms, "a mean equal to the worst means only one interval");
}

#[test]
fn a_new_window_starts_clean() {
    let mut m = meter();
    let t0 = Instant::now();
    m.record(t0);
    m.record(t0 + Duration::from_millis(500)); // 500 ms stall in window 1
    let first = m.record(t0 + Duration::from_millis(1100)).expect("window 1 closes");
    assert!(first.worst_ms >= 499.0, "window 1 missed its own stall: {}", first.worst_ms);

    // Window 2 renders smoothly all the way through: its worst frame must be
    // ~16 ms, not window 1's 600 ms stall carried over.
    let mut second = None;
    let mut t = 1100;
    while second.is_none() && t < 3000 {
        t += 16;
        second = m.record(t0 + Duration::from_millis(t));
    }
    let second = second.expect("window 2 closes");
    assert!(second.worst_ms < 40.0, "window 2 inherited window 1's stall: {}", second.worst_ms);
}

#[test]
fn the_mean_is_over_intervals_not_over_frames() {
    // Two frames 2 s apart = ONE interval of 2 s. Dividing by the frame count
    // instead would report 1 s and quietly halve every frame time.
    let mut m = meter();
    let t0 = Instant::now();
    m.record(t0);
    let s = m.record(t0 + Duration::from_secs(2)).expect("window closes");
    assert!((s.frame_ms - 2000.0).abs() < 1.0, "frame_ms was {}", s.frame_ms);
    assert!((s.worst_ms - 2000.0).abs() < 1.0, "worst_ms was {}", s.worst_ms);
}

#[test]
fn a_backwards_clock_does_not_panic() {
    let mut m = meter();
    let t0 = Instant::now() + Duration::from_secs(10);
    m.record(t0);
    m.record(t0 - Duration::from_secs(5));
    let _ = m.record(t0 + Duration::from_secs(2));
}

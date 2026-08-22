use super::*;
use rodio::buffer::SamplesBuffer;
use std::num::NonZero;
use std::sync::atomic::AtomicBool;

#[test]
fn test_tapped_source_passes_through() {
    let samples: Vec<f32> = vec![0.5, 0.75, -0.5, -0.25, 0.0];
    let source = SamplesBuffer::new(
        NonZero::new(1u16).unwrap(),
        NonZero::new(44100u32).unwrap(),
        samples.clone(),
    );

    let ring_buffer = Arc::new(RingBuffer::new(16));
    let enabled = Arc::new(AtomicBool::new(true));

    let tapped = TappedSource::new(source, ring_buffer, enabled);
    let output: Vec<f32> = tapped.collect();

    // Samples should pass through unchanged
    assert_eq!(output, samples);
}

#[test]
fn test_tapped_source_fills_ring_buffer() {
    let samples: Vec<f32> = vec![1.0, 0.0, -1.0];
    let source = SamplesBuffer::new(
        NonZero::new(1u16).unwrap(),
        NonZero::new(44100u32).unwrap(),
        samples,
    );

    let ring_buffer = Arc::new(RingBuffer::new(16));
    let enabled = Arc::new(AtomicBool::new(true));

    let tapped = TappedSource::new(source, ring_buffer.clone(), enabled);
    let _: Vec<f32> = tapped.collect();

    // Check ring buffer received samples directly (f32 already normalized)
    let mut snapshot = [0.0f32; 3];
    ring_buffer.snapshot(&mut snapshot);

    assert!((snapshot[0] - 1.0).abs() < 0.001);
    assert!((snapshot[1] - 0.0).abs() < 0.001);
    assert!((snapshot[2] - (-1.0)).abs() < 0.001);
}

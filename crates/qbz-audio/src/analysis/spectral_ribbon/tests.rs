use super::SpectralAnalyzer;

#[test]
fn spectral_analyzer_returns_expected_band_count() {
    let mut analyzer = SpectralAnalyzer::new(48_000, 1024, 64, 24, 0.8);
    let frame = vec![0.0f32; 1024];
    let _ = analyzer.process_audio_frame(&frame, 48_000);
    assert_eq!(analyzer.get_latest_bands().len(), 64);
}

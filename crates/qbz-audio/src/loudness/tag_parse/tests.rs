use super::*;

#[test]
fn test_parse_gain_value_formats() {
    // Standard format: "-6.54 dB"
    assert!(
        (parse_gain_value(&Value::String("-6.54 dB".to_string())).unwrap() - (-6.54)).abs()
            < 0.001
    );
    // Without dB suffix
    assert!(
        (parse_gain_value(&Value::String("-6.54".to_string())).unwrap() - (-6.54)).abs() < 0.001
    );
    // Positive
    assert!(
        (parse_gain_value(&Value::String("+3.21 dB".to_string())).unwrap() - 3.21).abs() < 0.001
    );
    // Float value
    assert!((parse_gain_value(&Value::Float(-6.54)).unwrap() - (-6.54)).abs() < 0.001);
}

#[test]
fn test_parse_peak_value() {
    assert!(
        (parse_peak_value(&Value::String("0.988553".to_string())).unwrap() - 0.988553).abs()
            < 0.0001
    );
    assert!((parse_peak_value(&Value::Float(0.95)).unwrap() - 0.95).abs() < 0.001);
}

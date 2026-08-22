//! Slice 7 — pure string helpers, no Slint dependency: manual node.name
//! validation/classification and the sample-rate label formatter.

/// Validate a manually-pasted node.name (escape hatch). 1:1 with the Tauri
/// `validateNodeName`.
pub(super) fn validate_node_name(name: &str) -> bool {
    let t = name.trim();
    !t.is_empty() && (t.contains("alsa_output") || t.contains("alsa_input"))
}

pub(super) fn detect_dac_type(name: &str) -> &'static str {
    let l = name.to_lowercase();
    if l.contains("usb-") || l.contains(".usb") {
        "usb"
    } else if l.contains("pci-") || l.contains(".pci") {
        "pci"
    } else if l.contains("bluez") || l.contains("bluetooth") {
        "bluetooth"
    } else if l.contains("virtual") || l.contains("null") || l.contains("dummy") {
        "virtual"
    } else {
        "unknown"
    }
}

/// "44.1 / 96 / 192 kHz" from a rate list (kHz, .1 only when non-integer).
pub(super) fn format_rates(rates: &[u32]) -> String {
    if rates.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = rates
        .iter()
        .map(|&r| {
            if r % 1000 == 0 {
                format!("{}", r / 1000)
            } else {
                format!("{:.1}", r as f64 / 1000.0)
            }
        })
        .collect();
    format!("{} kHz", parts.join(" / "))
}

#[cfg(test)]
mod slice7_tests {
    use super::*;

    #[test]
    fn validates_node_names_like_tauri() {
        assert!(validate_node_name("alsa_output.usb-Cambridge-00.analog-stereo"));
        assert!(validate_node_name("alsa_input.pci-0000_00.analog-stereo"));
        assert!(!validate_node_name(""));
        assert!(!validate_node_name("   "));
        assert!(!validate_node_name("bluez_output.AA_BB"));
    }

    #[test]
    fn detects_dac_type() {
        assert_eq!(detect_dac_type("alsa_output.usb-Cambridge-00.analog-stereo"), "usb");
        assert_eq!(detect_dac_type("alsa_output.pci-0000_00_1f.3.analog-stereo"), "pci");
        assert_eq!(detect_dac_type("bluez_output.AA"), "bluetooth");
        assert_eq!(detect_dac_type("alsa_output.virtual-dummy"), "virtual");
        assert_eq!(detect_dac_type("something.else"), "unknown");
    }

    #[test]
    fn formats_rates_khz() {
        assert_eq!(format_rates(&[44100, 96000, 192000]), "44.1 / 96 / 192 kHz");
        assert_eq!(format_rates(&[]), "");
    }
}

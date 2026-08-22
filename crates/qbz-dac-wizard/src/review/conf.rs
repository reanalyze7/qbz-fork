//! Slice 10 (review-and-apply) — pure string generation, no Slint types:
//! filename slugging + the three config snippets (pipewire / pulse /
//! wireplumber) written per DAC.

/// A short, filename-safe DAC name: slug of the description, else the node.name.
pub(crate) fn short_name(name: &str, node_name: &str) -> String {
    let slug = slugify(name);
    if !slug.is_empty() {
        return slug;
    }
    let nslug = slugify(node_name);
    if nslug.is_empty() {
        "dac".to_string()
    } else {
        nslug
    }
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn rates_list(rates: &[u32]) -> String {
    if rates.is_empty() {
        "44100 48000 88200 96000 176400 192000".to_string()
    } else {
        rates
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub(crate) fn pipewire_conf(short: &str, rates: &[u32]) -> String {
    let rates = rates_list(rates);
    [
        "mkdir -p ~/.config/pipewire/pipewire.conf.d".to_string(),
        format!("cat > ~/.config/pipewire/pipewire.conf.d/99-qbz-dac-{short}.conf << 'EOF'"),
        "# QBZ DAC Setup - Sample Rate Switching".to_string(),
        "context.properties = {".to_string(),
        format!("  default.clock.allowed-rates = [ {rates} ]"),
        "}".to_string(),
        "EOF".to_string(),
    ]
    .join("\n")
}

pub(crate) fn pulse_conf(short: &str) -> String {
    [
        "mkdir -p ~/.config/pipewire/client.conf.d".to_string(),
        format!("cat > ~/.config/pipewire/client.conf.d/99-qbz-bitperfect-{short}.conf << 'EOF'"),
        "# QBZ DAC Setup - Per-App Bit-Perfect".to_string(),
        "stream.rules = [".to_string(),
        "  {".to_string(),
        "    matches = [".to_string(),
        "      { application.process.binary = \"qbz\" }".to_string(),
        "      { application.name = \"PipeWire ALSA [qbz]\" }".to_string(),
        "    ]".to_string(),
        "    actions = { update-props = { resample.disable = true, channelmix.disable = true } }"
            .to_string(),
        "  }".to_string(),
        "]".to_string(),
        "EOF".to_string(),
    ]
    .join("\n")
}

pub(crate) fn wireplumber_conf(short: &str, node_name: &str, rates: &[u32], description: &str) -> String {
    let rates = rates_list(rates);
    [
        "mkdir -p ~/.config/wireplumber/wireplumber.conf.d".to_string(),
        format!("cat > ~/.config/wireplumber/wireplumber.conf.d/99-qbz-dac-{short}.conf << 'EOF'"),
        format!("# QBZ DAC Setup - {description}"),
        "monitor.alsa.rules = [".to_string(),
        "  {".to_string(),
        "    matches = [".to_string(),
        format!("      {{ node.name = \"{node_name}\", media.class = \"Audio/Sink\" }}"),
        "    ]".to_string(),
        "    actions = {".to_string(),
        "      update-props = {".to_string(),
        format!("        audio.allowed-rates = [ {rates} ]"),
        "        resample.disable = true".to_string(),
        "        channelmix.disable = true".to_string(),
        "      }".to_string(),
        "    }".to_string(),
        "  }".to_string(),
        "]".to_string(),
        "EOF".to_string(),
    ]
    .join("\n")
}

#[cfg(test)]
mod slice10_tests {
    use super::*;

    #[test]
    fn slugifies_descriptions() {
        assert_eq!(slugify("DacMagic Plus Analog Stereo"), "dacmagic-plus-analog-stereo");
        assert_eq!(slugify("Built-in Audio Analog Stereo"), "built-in-audio-analog-stereo");
        assert_eq!(slugify("  weird__name!! "), "weird-name");
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn wireplumber_conf_pins_node_and_rates() {
        let c = wireplumber_conf("dacmagic", "alsa_output.usb-x.analog-stereo", &[44100, 192000], "DacMagic");
        assert!(c.contains("node.name = \"alsa_output.usb-x.analog-stereo\""));
        assert!(c.contains("audio.allowed-rates = [ 44100 192000 ]"));
        assert!(c.contains("99-qbz-dac-dacmagic.conf"));
        assert!(c.contains("resample.disable = true"));
    }
}

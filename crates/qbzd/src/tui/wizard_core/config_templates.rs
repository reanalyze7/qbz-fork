pub(super) fn rates_list(rates: &[u32]) -> String {
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

pub(super) fn pipewire_conf(short: &str, rates: &[u32]) -> String {
    let rates = rates_list(rates);
    [
        "mkdir -p ~/.config/pipewire/pipewire.conf.d".to_string(),
        format!("cat > ~/.config/pipewire/pipewire.conf.d/99-qbz-dac-{short}.conf << 'EOF'"),
        "# Qoqobuz DAC Setup - Sample Rate Switching".to_string(),
        "context.properties = {".to_string(),
        format!("  default.clock.allowed-rates = [ {rates} ]"),
        "}".to_string(),
        "EOF".to_string(),
    ]
    .join("\n")
}

pub(super) fn pulse_conf(short: &str) -> String {
    [
        "mkdir -p ~/.config/pipewire/client.conf.d".to_string(),
        format!("cat > ~/.config/pipewire/client.conf.d/99-qbz-bitperfect-{short}.conf << 'EOF'"),
        "# Qoqobuz DAC Setup - Per-App Bit-Perfect".to_string(),
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

pub(super) fn wireplumber_conf(short: &str, node_name: &str, rates: &[u32], description: &str) -> String {
    let rates = rates_list(rates);
    [
        "mkdir -p ~/.config/wireplumber/wireplumber.conf.d".to_string(),
        format!("cat > ~/.config/wireplumber/wireplumber.conf.d/99-qbz-dac-{short}.conf << 'EOF'"),
        format!("# Qoqobuz DAC Setup - {description}"),
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

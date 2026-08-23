use qbz_audio::{AudioStackHealth, Distro, InitSystem, NegotiatedRate};

use super::config_gen::{slugify, DacConfigData};
use super::config_templates::wireplumber_conf;
use super::detect::{detect_dac_type, format_rates, validate_node_name};
use super::remediate::{reference_commands, remediations};
use super::test_seeds::{negotiated_label, seed_for_rate_depth};

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

#[test]
fn full_block_and_paths_cover_the_three_files() {
    let cfg = DacConfigData {
        name: "DacMagic".to_string(),
        node_name: "alsa_output.usb-x.analog-stereo".to_string(),
        pipewire_conf: "PW".to_string(),
        pulse_conf: "PULSE".to_string(),
        wireplumber_conf: "WP".to_string(),
    };
    assert_eq!(cfg.short(), "dacmagic");
    let block = cfg.full_block();
    assert!(block.contains("PW") && block.contains("PULSE") && block.contains("WP"));
    let paths = cfg.target_paths();
    assert_eq!(paths.len(), 3);
    assert!(paths[0].contains("pipewire.conf.d/99-qbz-dac-dacmagic.conf"));
    assert!(paths[1].contains("client.conf.d/99-qbz-bitperfect-dacmagic.conf"));
    assert!(paths[2].contains("wireplumber.conf.d/99-qbz-dac-dacmagic.conf"));
}

#[test]
fn seed_lookup_matches_known_reference_rates() {
    // 24/192 → Toto "Africa"; 16/44100 → George Harrison.
    assert_eq!(seed_for_rate_depth(192000, 24).map(|s| s.title), Some("Africa"));
    assert_eq!(seed_for_rate_depth(44100, 16).map(|s| s.title), Some("My Sweet Lord"));
    // The two 44.1 seeds differ only by depth.
    assert_eq!(seed_for_rate_depth(44100, 24).map(|s| s.title), Some("LUNCH"));
    // An off-grid rate matches nothing.
    assert!(seed_for_rate_depth(48000, 24).is_none());
}

#[test]
fn remediations_nixos_collapses_to_one_config_block() {
    let unhealthy = AudioStackHealth {
        wireplumber_active: false,
        has_pw_dump: false,
        cpal_sees_pipewire: false,
        has_pactl: false,
        any_devices: false,
    };
    let r = remediations(unhealthy, Distro::NixOS, InitSystem::Systemd);
    assert_eq!(r.len(), 1);
    assert!(r[0].1.contains("services.pipewire"));
    assert!(r[0].1.contains("nixos-rebuild switch"));
}

#[test]
fn remediations_debian_names_the_alsa_bridge_and_is_init_aware() {
    let missing_bridge = AudioStackHealth {
        wireplumber_active: true,
        has_pw_dump: true,
        cpal_sees_pipewire: false, // the Ubuntu empty-list bug
        has_pactl: true,
        any_devices: true,
    };
    let r = remediations(missing_bridge, Distro::Debian, InitSystem::Systemd);
    assert!(r.iter().any(|(_, cmd)| cmd == "sudo apt install pipewire-alsa"));
    // needs_restart flipped → an init-aware systemd restart block is appended.
    assert!(r.iter().any(|(_, cmd)| cmd.contains("systemctl --user restart")));
}

#[test]
fn reference_commands_used_in_sandbox_full_stack() {
    let r = reference_commands(Distro::Debian, InitSystem::Systemd);
    assert_eq!(r.len(), 2);
    assert!(r[0].1.contains("pipewire-alsa"), "full stack must include the ALSA bridge");
    assert!(r[1].1.contains("systemctl --user restart"));
}

#[test]
fn negotiated_label_shows_rate_format_channels() {
    let n = NegotiatedRate { sample_rate: 192000, format: "S32_LE".to_string(), channels: 2 };
    assert_eq!(negotiated_label(&n), "DAC: 192 kHz · S32_LE · 2 ch");
}

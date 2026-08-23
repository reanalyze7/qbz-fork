// ── check step: audio-stack remediations (per distro / init) ────────────────
use qbz_audio::{AudioStackHealth, Distro, InitSystem};

use super::remediate_pkgs::{full_stack_pkgs, install, install_reinstall, pkg_pulse, pkg_pw_tools};

/// (caption, copy-paste command) per missing probe, for the given distro.
///
/// Service/restart commands are INIT-SYSTEM aware per distro (OpenRC on Gentoo,
/// runit on Void, systemd elsewhere), mirroring the Tauri DistroSelector
/// `restartCommands`. Installs and the restart are kept as separate blocks so
/// the multi-line Gentoo guidance never gets `&&`-joined.
pub fn remediations(h: AudioStackHealth, d: Distro, init: InitSystem) -> Vec<(String, String)> {
    // NixOS is declarative: you don't `apt/pacman install` pieces — you enable
    // the PipeWire module and rebuild. So collapse all the missing pieces into
    // one config block instead of per-package commands.
    if d == Distro::NixOS {
        if h.is_ready() {
            return Vec::new();
        }
        return vec![(
            "Enable PipeWire in your NixOS configuration".to_string(),
            NIXOS_PIPEWIRE_BLOCK.to_string(),
        )];
    }

    let mut out = Vec::new();
    let mut needs_restart = false;
    if !h.has_pw_dump {
        out.push((
            "Install the PipeWire tools (pw-dump)".to_string(),
            install(d, pkg_pw_tools(d)),
        ));
        needs_restart = true;
    }
    if !h.cpal_sees_pipewire {
        // THE Ubuntu no-list / no-playback bug: the ALSA->PipeWire bridge PCM.
        out.push((
            "Install the ALSA bridge so playback can reach PipeWire".to_string(),
            install(d, "pipewire-alsa"),
        ));
        needs_restart = true;
    }
    if !h.has_pactl {
        out.push((
            "Install the Pulse compatibility tools (optional fallback)".to_string(),
            install(d, pkg_pulse(d)),
        ));
        needs_restart = true;
    }
    if !h.any_devices {
        out.push((
            "No sinks detected — reinstall the ALSA UCM profiles, then reboot".to_string(),
            install_reinstall(d, "alsa-ucm-conf"),
        ));
    }
    // WirePlumber down, or we just installed something → (re)start the stack
    // with the ACTUAL init system running on this machine (not guessed from the
    // distro — Gentoo+systemd and Gentoo+OpenRC must differ).
    if !h.wireplumber_active || needs_restart {
        out.push((
            "(Re)start the PipeWire audio services".to_string(),
            restart_cmd(init).to_string(),
        ));
    }
    out
}

/// Init-system-aware "(re)start the audio services" command. PipeWire is a
/// user-session service, so only systemd has a first-class `--user` restart;
/// the others either use their user-service supervisor or a re-login.
pub fn restart_cmd(init: InitSystem) -> &'static str {
    match init {
        InitSystem::Systemd => "systemctl --user restart pipewire pipewire-pulse wireplumber",
        InitSystem::OpenRc => {
            "# OpenRC: PipeWire runs in your user session, not as an OpenRC service.\n\
             # Log out and back in to restart it."
        }
        InitSystem::Runit => {
            "sv restart pipewire wireplumber   # if set up as runit user services; otherwise log out and back in"
        }
        InitSystem::S6 => "# s6: restart via your supervision tree, or log out and back in",
        InitSystem::Dinit => "dinitctl restart pipewire wireplumber   # or log out and back in",
        InitSystem::Unknown => "# Restart PipeWire via your init system, or log out and back in",
    }
}

pub(super) const NIXOS_PIPEWIRE_BLOCK: &str = "# /etc/nixos/configuration.nix:\n\
     services.pipewire = {\n\
     \u{20}\u{20}enable = true;\n\
     \u{20}\u{20}alsa.enable = true;\n\
     \u{20}\u{20}pulse.enable = true;\n\
     \u{20}\u{20}wireplumber.enable = true;\n\
     };\n\
     # then apply:\n\
     sudo nixos-rebuild switch";

/// Full reference setup commands for the chosen distro/init, shown when QBZ
/// can't probe the host (sandbox). Mirrors the Tauri DistroSelector, which
/// always showed per-distro install + restart commands (no probing).
pub fn reference_commands(d: Distro, init: InitSystem) -> Vec<(String, String)> {
    if d == Distro::NixOS {
        return vec![(
            "Enable PipeWire in your NixOS configuration".to_string(),
            NIXOS_PIPEWIRE_BLOCK.to_string(),
        )];
    }
    vec![
        (
            "Install the PipeWire audio stack".to_string(),
            install(d, full_stack_pkgs(d)),
        ),
        (
            "(Re)start the PipeWire audio services".to_string(),
            restart_cmd(init).to_string(),
        ),
    ]
}

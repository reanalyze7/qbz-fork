//! Slice 6 (check step) — pure command-string generation, no Slint dependency:
//! per-distro/init remediation commands (the day-to-day "probe found a gap,
//! here's the fix" list). Reference/install commands live in `install.rs`.

use qbz_audio::{AudioStackHealth, Distro, InitSystem};

use super::install::{install, install_reinstall, pkg_pulse, pkg_pw_tools, NIXOS_PIPEWIRE_BLOCK};

/// (caption, copy-paste command) per missing probe, for the given distro.
///
/// Service/restart commands are INIT-SYSTEM aware per distro (OpenRC on Gentoo,
/// runit on Void, systemd elsewhere), mirroring the Tauri DistroSelector
/// `restartCommands`. Installs and the restart are kept as separate blocks so
/// the multi-line Gentoo guidance never gets `&&`-joined.
pub(crate) fn remediations(h: AudioStackHealth, d: Distro, init: InitSystem) -> Vec<(String, String)> {
    // NixOS is declarative: you don't `apt/pacman install` pieces — you enable
    // the PipeWire module and rebuild. So collapse all the missing pieces into
    // one config block instead of per-package commands.
    if d == Distro::NixOS {
        if h.is_ready() {
            return Vec::new();
        }
        return vec![(
            qbz_i18n::t("Enable PipeWire in your NixOS configuration"),
            NIXOS_PIPEWIRE_BLOCK.to_string(),
        )];
    }

    let mut out = Vec::new();
    let mut needs_restart = false;
    if !h.has_pw_dump {
        out.push((
            qbz_i18n::t("Install the PipeWire tools (pw-dump)"),
            install(d, pkg_pw_tools(d)),
        ));
        needs_restart = true;
    }
    if !h.cpal_sees_pipewire {
        // THE Ubuntu no-list / no-playback bug: the ALSA->PipeWire bridge PCM.
        out.push((
            qbz_i18n::t("Install the ALSA bridge so playback can reach PipeWire"),
            install(d, "pipewire-alsa"),
        ));
        needs_restart = true;
    }
    if !h.has_pactl {
        out.push((
            qbz_i18n::t("Install the Pulse compatibility tools (optional fallback)"),
            install(d, pkg_pulse(d)),
        ));
        needs_restart = true;
    }
    if !h.any_devices {
        out.push((
            qbz_i18n::t("No sinks detected — reinstall the ALSA UCM profiles, then reboot"),
            install_reinstall(d, "alsa-ucm-conf"),
        ));
    }
    // WirePlumber down, or we just installed something → (re)start the stack
    // with the ACTUAL init system running on this machine (not guessed from the
    // distro — Gentoo+systemd and Gentoo+OpenRC must differ).
    if !h.wireplumber_active || needs_restart {
        out.push((
            qbz_i18n::t("(Re)start the PipeWire audio services"),
            restart_cmd(init).to_string(),
        ));
    }
    out
}

/// Init-system-aware "(re)start the audio services" command. PipeWire is a
/// user-session service, so only systemd has a first-class `--user` restart;
/// the others either use their user-service supervisor or a re-login.
pub(crate) fn restart_cmd(init: InitSystem) -> &'static str {
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

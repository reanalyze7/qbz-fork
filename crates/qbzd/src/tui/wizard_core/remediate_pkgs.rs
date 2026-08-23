use qbz_audio::Distro;

pub(super) fn pkg_pw_tools(d: Distro) -> &'static str {
    match d {
        // Debian-family (incl. antiX) ship pw-* in pipewire-bin.
        Distro::Debian | Distro::Antix => "pipewire-bin",
        Distro::Fedora => "pipewire-utils",
        // Arch (incl. Artix) / openSUSE / Gentoo / Void ship pw-* with pipewire.
        _ => "pipewire",
    }
}

pub(super) fn pkg_pulse(d: Distro) -> &'static str {
    match d {
        Distro::Debian | Distro::Antix => "pipewire-pulse pulseaudio-utils",
        Distro::Fedora => "pipewire-pulseaudio",
        _ => "pipewire-pulse",
    }
}

pub(super) fn install(d: Distro, pkgs: &str) -> String {
    match d {
        // Package manager is a property of the distro family, NOT the init.
        Distro::Debian | Distro::Antix => format!("sudo apt install {pkgs}"),
        Distro::Fedora => format!("sudo dnf install {pkgs}"),
        Distro::Arch | Distro::Artix => format!("sudo pacman -S {pkgs}"),
        Distro::OpenSuse => format!("sudo zypper install {pkgs}"),
        Distro::Gentoo => format!("sudo emerge {pkgs}   # package name may differ on Gentoo"),
        Distro::Void => format!("sudo xbps-install -S {pkgs}"),
        // NixOS is special-cased in remediations(); this is an unreached fallback.
        Distro::NixOS => format!("# NixOS: add to configuration.nix (see the PipeWire block) — {pkgs}"),
        Distro::Other => format!("Install with your package manager: {pkgs}"),
    }
}

pub(super) fn install_reinstall(d: Distro, pkg: &str) -> String {
    match d {
        Distro::Debian | Distro::Antix => format!("sudo apt install --reinstall {pkg}"),
        Distro::Fedora => format!("sudo dnf reinstall {pkg}"),
        _ => install(d, pkg),
    }
}

/// The full recommended package set (incl. `pipewire-alsa`, the bit the old
/// Tauri list omitted — the cause of the Ubuntu empty-list bug).
pub(super) fn full_stack_pkgs(d: Distro) -> &'static str {
    match d {
        Distro::Debian | Distro::Antix => {
            "pipewire pipewire-pulse pipewire-alsa wireplumber alsa-utils"
        }
        Distro::Fedora => "pipewire pipewire-pulseaudio pipewire-alsa wireplumber alsa-utils",
        Distro::Arch | Distro::Artix => {
            "pipewire pipewire-pulse pipewire-alsa wireplumber alsa-utils"
        }
        Distro::OpenSuse => "pipewire pipewire-pulseaudio pipewire-alsa wireplumber alsa-utils",
        Distro::Gentoo => "media-video/pipewire media-video/wireplumber media-sound/alsa-utils",
        Distro::Void => "pipewire wireplumber alsa-utils",
        Distro::NixOS => "",
        Distro::Other => "pipewire pipewire-pulse wireplumber alsa-utils",
    }
}

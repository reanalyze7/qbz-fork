use super::sandbox::{detect_sandbox, Sandbox};

/// Linux distribution family — drives the per-distro install commands. Order
/// matches the wizard's distro dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distro {
    Debian,
    /// Debian-based but systemd-free (sysVinit/runit).
    Antix,
    Fedora,
    Arch,
    /// Arch-based but systemd-free (OpenRC/runit/s6/dinit).
    Artix,
    OpenSuse,
    Gentoo,
    Void,
    /// Declarative — packages live in configuration.nix, init is systemd.
    NixOS,
    Other,
}

impl Distro {
    /// Dropdown order (index = position here). `Other` stays last.
    pub const ALL: [Distro; 10] = [
        Distro::Debian,
        Distro::Antix,
        Distro::Fedora,
        Distro::Arch,
        Distro::Artix,
        Distro::OpenSuse,
        Distro::Gentoo,
        Distro::Void,
        Distro::NixOS,
        Distro::Other,
    ];

    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&d| d == self)
            .unwrap_or(Self::ALL.len() - 1)
    }

    /// Human label for the dropdown (mirrors the Tauri DistroSelector, plus the
    /// systemd-free families called out so the init-aware commands make sense).
    pub fn label(self) -> &'static str {
        match self {
            Distro::Debian => "Ubuntu / Debian / Mint / Pop!_OS",
            Distro::Antix => "antiX (systemd-free Debian)",
            Distro::Fedora => "Fedora / RHEL",
            Distro::Arch => "Arch / Manjaro / EndeavourOS",
            Distro::Artix => "Artix (systemd-free Arch)",
            Distro::OpenSuse => "openSUSE",
            Distro::Gentoo => "Gentoo / Funtoo",
            Distro::Void => "Void Linux",
            Distro::NixOS => "NixOS",
            Distro::Other => "Other",
        }
    }
}

/// Detect the distro from the HOST `os-release`, defaulting to `Other`.
///
/// Inside a sandbox the plain `/etc/os-release` is the runtime's (Flatpak
/// freedesktop-sdk) or the snap base's, NOT the user's distro — so read the
/// host-exposed path first: Flatpak guarantees `/run/host/os-release`, Snap
/// mounts the host root at `/var/lib/snapd/hostfs`. Falls back to `/etc`.
pub fn detect_distro() -> Distro {
    let host_path = match detect_sandbox() {
        Sandbox::Flatpak => Some("/run/host/os-release"),
        Sandbox::Snap => Some("/var/lib/snapd/hostfs/etc/os-release"),
        Sandbox::None => None,
    };
    let content = host_path
        .and_then(|p| std::fs::read_to_string(p).ok())
        .or_else(|| std::fs::read_to_string("/etc/os-release").ok());
    content.map(|c| parse_distro(&c)).unwrap_or(Distro::Other)
}

/// Pure `/etc/os-release` classifier (testable). Reads `ID` then `ID_LIKE`.
pub(super) fn parse_distro(os_release: &str) -> Distro {
    let mut id = String::new();
    let mut id_like = String::new();
    for line in os_release.lines() {
        // os-release values may be bare, double-quoted, or single-quoted
        // (Gentoo uses single quotes), so strip both quote styles.
        if let Some(v) = line.strip_prefix("ID=") {
            id = v.trim().trim_matches(|c| c == '"' || c == '\'').to_lowercase();
        } else if let Some(v) = line.strip_prefix("ID_LIKE=") {
            id_like = v.trim().trim_matches(|c| c == '"' || c == '\'').to_lowercase();
        }
    }
    let hay = format!("{} {}", id, id_like);
    let has = |needle: &str| hay.contains(needle);
    // Systemd-free derivatives MUST be matched before their parent family —
    // antiX has ID_LIKE=debian and Artix has ID_LIKE=arch, so the generic
    // checks below would otherwise swallow them and emit systemd commands.
    if has("antix") {
        Distro::Antix
    } else if has("artix") {
        Distro::Artix
    } else if has("nixos") {
        Distro::NixOS
    } else if has("ubuntu") || has("debian") || has("mint") || has("pop") {
        Distro::Debian
    } else if has("fedora") || has("rhel") || has("centos") {
        Distro::Fedora
    } else if has("arch") || has("manjaro") || has("endeavour") {
        Distro::Arch
    } else if has("suse") {
        Distro::OpenSuse
    } else if has("gentoo") {
        Distro::Gentoo
    } else if has("void") {
        Distro::Void
    } else {
        Distro::Other
    }
}

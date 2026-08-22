use super::sandbox::{detect_sandbox, Sandbox};

/// The running init / service manager. Detected at RUNTIME — it is orthogonal
/// to the distro (Gentoo runs OpenRC *or* systemd; Debian runs systemd or
/// sysVinit/runit on antiX), so service commands must key off this, not the
/// distro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitSystem {
    Systemd,
    OpenRc,
    Runit,
    S6,
    Dinit,
    Unknown,
}

impl InitSystem {
    /// Dropdown order (index = position here). `Unknown` stays last.
    pub const ALL: [InitSystem; 6] = [
        InitSystem::Systemd,
        InitSystem::OpenRc,
        InitSystem::Runit,
        InitSystem::S6,
        InitSystem::Dinit,
        InitSystem::Unknown,
    ];

    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&i| i == self)
            .unwrap_or(Self::ALL.len() - 1)
    }

    pub fn label(self) -> &'static str {
        match self {
            InitSystem::Systemd => "systemd",
            InitSystem::OpenRc => "OpenRC",
            InitSystem::Runit => "runit",
            InitSystem::S6 => "s6",
            InitSystem::Dinit => "dinit",
            InitSystem::Unknown => "Other / unknown",
        }
    }
}

/// Detect the running init system. The `/run/systemd/system` check is the
/// canonical `sd_booted()` test; the others mirror each supervisor's runtime
/// dir, with a `/proc/1/comm` fallback.
///
/// In a sandbox these all reflect the SANDBOX, not the host, so we return
/// `Unknown` and let the wizard's init override decide.
pub fn detect_init() -> InitSystem {
    if detect_sandbox() != Sandbox::None {
        return InitSystem::Unknown;
    }
    use std::path::Path;
    if Path::new("/run/systemd/system").exists() {
        return InitSystem::Systemd;
    }
    if Path::new("/run/openrc").exists() {
        return InitSystem::OpenRc;
    }
    if Path::new("/run/runit").exists() || Path::new("/etc/runit").exists() {
        return InitSystem::Runit;
    }
    if Path::new("/run/s6-rc").exists() || Path::new("/run/s6").exists() {
        return InitSystem::S6;
    }
    if Path::new("/run/dinitctl").exists() {
        return InitSystem::Dinit;
    }
    std::fs::read_to_string("/proc/1/comm")
        .map(|c| parse_init_from_comm(c.trim()))
        .unwrap_or(InitSystem::Unknown)
}

/// Pure classifier for PID 1's `comm` (testable fallback path).
pub(super) fn parse_init_from_comm(comm: &str) -> InitSystem {
    match comm {
        "systemd" => InitSystem::Systemd,
        "openrc-init" | "openrc" => InitSystem::OpenRc,
        "runit" | "runsvdir" | "runit-init" => InitSystem::Runit,
        "s6-svscan" | "s6-linux-init" => InitSystem::S6,
        "dinit" => InitSystem::Dinit,
        _ => InitSystem::Unknown,
    }
}

/// App packaging sandbox. Inside one, host files (`/etc/os-release`, `/run`,
/// `/proc/1`) reflect the SANDBOX/runtime, not the user's host — so host
/// detection must read the host-exposed paths, and init detection can't be
/// trusted (it falls back to the manual override).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sandbox {
    None,
    Flatpak,
    Snap,
}

/// Detect the packaging sandbox: Flatpak exposes `/.flatpak-info`, Snap sets `$SNAP`.
pub fn detect_sandbox() -> Sandbox {
    if std::path::Path::new("/.flatpak-info").exists() {
        Sandbox::Flatpak
    } else if std::env::var_os("SNAP").is_some() {
        Sandbox::Snap
    } else {
        Sandbox::None
    }
}

//! Framework-agnostic graphics environment detection.
//!
//! Detects the host display server, GPU vendors, desktop, and VM status.
//! Consumed by the diagnostics panel. (`use serde::Serialize` stays — the
//! `Environment` struct still derives it.)

mod gpu;
mod vm;

use serde::Serialize;

pub use gpu::detect_gpu_name;
use gpu::{is_amd_gpu, is_intel_gpu, is_nvidia_gpu};
use vm::is_virtual_machine;

/// Detected environment information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Environment {
    pub display_server: String,
    pub gpu_nvidia: bool,
    pub gpu_amd: bool,
    pub gpu_intel: bool,
    pub gpu_name: String,
    pub desktop: String,
    pub is_vm: bool,
}

pub fn detect_environment() -> Environment {
    let display_server = detect_display_server();
    let gpu_nvidia = is_nvidia_gpu();
    let gpu_amd = is_amd_gpu();
    let gpu_intel = is_intel_gpu();
    let gpu_name = detect_gpu_name(gpu_nvidia, gpu_amd, gpu_intel);
    let desktop = detect_desktop();
    let is_vm = is_virtual_machine();

    Environment {
        display_server,
        gpu_nvidia,
        gpu_amd,
        gpu_intel,
        gpu_name,
        desktop,
        is_vm,
    }
}

fn detect_display_server() -> String {
    let is_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland");

    if is_wayland {
        "Wayland".to_string()
    } else {
        "X11".to_string()
    }
}

fn detect_desktop() -> String {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let session = std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default();
    let de = std::env::var("DESKTOP_SESSION").unwrap_or_default();

    if !desktop.is_empty() {
        desktop
    } else if !session.is_empty() {
        session
    } else if !de.is_empty() {
        de
    } else {
        "Unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_name_combines_hybrid_vendors() {
        let name = detect_gpu_name(true, false, true);

        assert!(name.contains("NVIDIA"));
        assert!(name.contains("Intel"));
        assert!(name.contains(" + "));
    }
}

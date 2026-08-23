use super::detect::{detect_install_method, detect_kernel_version, detect_loaded_lib_version, read_os_release};
use super::types::SystemInfo;

/// Build the system info snapshot. Pure + infallible.
///
/// Faithful port of `v2_get_system_info` (Tauri) minus the Ok-wrap.
pub fn system_info() -> SystemInfo {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let (install_method, flatpak_runtime, flatpak_runtime_version) = detect_install_method();
    let osr = read_os_release();

    SystemInfo {
        os,
        arch,
        kernel_version: detect_kernel_version(),
        distro_id: osr.get("ID").cloned(),
        distro_version_id: osr.get("VERSION_ID").cloned(),
        distro_pretty_name: osr
            .get("PRETTY_NAME")
            .cloned()
            .or_else(|| osr.get("NAME").cloned()),
        install_method,
        flatpak_runtime,
        flatpak_runtime_version,
        // Runtime shared-library versions, parsed from /proc/self/maps.
        webkit2gtk_version: detect_loaded_lib_version("libwebkit2gtk-4.1"),
        gtk_version: detect_loaded_lib_version("libgtk-3")
            .or_else(|| detect_loaded_lib_version("libgtk-4")),
        glibc_version: detect_loaded_lib_version("libc"),
        alsa_version: detect_loaded_lib_version("libasound"),
        pipewire_version: detect_loaded_lib_version("libpipewire-0.3"),
        pulseaudio_version: detect_loaded_lib_version("libpulse"),
    }
}

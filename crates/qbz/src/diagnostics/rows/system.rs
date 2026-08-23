use qbz_app::diagnostics::SystemInfo;

use super::helpers::{opt, row};

pub(super) fn build_system_rows(s: &SystemInfo) -> Vec<crate::DiagRow> {
    let mut rows = vec![
        row("OS", "—", &s.os, 0),
        row("Arch", "—", &s.arch, 0),
        row("Kernel", "—", &opt(&s.kernel_version), 0),
        row("Distro", "—", &opt(&s.distro_pretty_name), 0),
        row("Distro ID", "—", &opt(&s.distro_id), 0),
        row("Distro Version", "—", &opt(&s.distro_version_id), 0),
        row("Install Method", "—", &s.install_method, 0),
    ];
    if let Some(runtime) = &s.flatpak_runtime {
        rows.push(row(
            "Flatpak Runtime",
            "—",
            &format!("{} {}", runtime, opt(&s.flatpak_runtime_version)),
            0,
        ));
    }
    rows.push(row("WebKit2GTK", "—", &opt(&s.webkit2gtk_version), 0));
    rows.push(row("GTK", "—", &opt(&s.gtk_version), 0));
    rows.push(row("glibc", "—", &opt(&s.glibc_version), 0));
    rows.push(row("ALSA", "—", &opt(&s.alsa_version), 0));
    rows.push(row("PipeWire", "—", &opt(&s.pipewire_version), 0));
    rows.push(row("PulseAudio", "—", &opt(&s.pulseaudio_version), 0));
    rows
}

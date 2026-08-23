//! The `## System` markdown section.

use qbz_app::diagnostics::SystemInfo;

use super::super::rows::opt;
use super::md_line;

pub(super) fn write_section(out: &mut String, sys: &SystemInfo) {
    out.push_str("\n## System\n\n");
    md_line(out, "OS", &sys.os);
    md_line(out, "Arch", &sys.arch);
    md_line(out, "Kernel", &opt(&sys.kernel_version));
    md_line(out, "Distro", &opt(&sys.distro_pretty_name));
    md_line(out, "Distro ID", &opt(&sys.distro_id));
    md_line(out, "Distro Version", &opt(&sys.distro_version_id));
    md_line(out, "Install Method", &sys.install_method);
    if let Some(rt) = &sys.flatpak_runtime {
        md_line(
            out,
            "Flatpak Runtime",
            &format!("{} {}", rt, opt(&sys.flatpak_runtime_version)),
        );
    }
    md_line(out, "WebKit2GTK", &opt(&sys.webkit2gtk_version));
    md_line(out, "GTK", &opt(&sys.gtk_version));
    md_line(out, "glibc", &opt(&sys.glibc_version));
    md_line(out, "ALSA", &opt(&sys.alsa_version));
    md_line(out, "PipeWire", &opt(&sys.pipewire_version));
    md_line(out, "PulseAudio", &opt(&sys.pulseaudio_version));
}

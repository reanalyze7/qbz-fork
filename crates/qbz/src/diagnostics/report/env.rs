//! The `## Environment` markdown section.

use qbz_app::diagnostics::RuntimeDiagnostics;

use super::super::rows::opt;
use super::md_line;

pub(super) fn write_section(out: &mut String, d: &RuntimeDiagnostics) {
    out.push_str("\n## Environment\n\n");
    md_line(
        out,
        "WEBKIT_DISABLE_DMABUF_RENDERER",
        &opt(&d.env_webkit_disable_dmabuf),
    );
    md_line(
        out,
        "WEBKIT_DISABLE_COMPOSITING_MODE",
        &opt(&d.env_webkit_disable_compositing),
    );
    md_line(out, "GDK_BACKEND", &opt(&d.env_gdk_backend));
    md_line(out, "GSK_RENDERER", &opt(&d.env_gsk_renderer));
    md_line(
        out,
        "LIBGL_ALWAYS_SOFTWARE",
        &opt(&d.env_libgl_always_software),
    );
    md_line(out, "WAYLAND_DISPLAY", &opt(&d.env_wayland_display));
    md_line(out, "XDG_SESSION_TYPE", &opt(&d.env_xdg_session_type));
}

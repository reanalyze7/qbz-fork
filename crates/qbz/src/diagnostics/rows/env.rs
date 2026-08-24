use qbz_app::diagnostics::RuntimeDiagnostics;

use super::helpers::{opt, row};

pub(crate) fn build_env_rows(d: &RuntimeDiagnostics) -> Vec<crate::DiagRow> {
    vec![
        row(
            "WEBKIT_DISABLE_DMABUF_RENDERER",
            "—",
            &opt(&d.env_webkit_disable_dmabuf),
            0,
        ),
        row(
            "WEBKIT_DISABLE_COMPOSITING_MODE",
            "—",
            &opt(&d.env_webkit_disable_compositing),
            0,
        ),
        row("GDK_BACKEND", "—", &opt(&d.env_gdk_backend), 0),
        row("GSK_RENDERER", "—", &opt(&d.env_gsk_renderer), 0),
        row(
            "LIBGL_ALWAYS_SOFTWARE",
            "—",
            &opt(&d.env_libgl_always_software),
            0,
        ),
        row("WAYLAND_DISPLAY", "—", &opt(&d.env_wayland_display), 0),
        row("XDG_SESSION_TYPE", "—", &opt(&d.env_xdg_session_type), 0),
    ]
}

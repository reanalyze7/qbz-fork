//! The `## Graphics` markdown section (saved + runtime).

use qbz_app::diagnostics::RuntimeDiagnostics;

use super::super::rows::{opt, yn};
use super::md_line;

pub(super) fn write_section(out: &mut String, d: &RuntimeDiagnostics) {
    out.push_str("\n## Graphics\n\n");
    let compositing = if d.env_webkit_disable_compositing.as_deref() == Some("1") {
        "DISABLED"
    } else {
        "ENABLED"
    };
    let dmabuf = if d.env_webkit_disable_dmabuf.as_deref() == Some("1") {
        "DISABLED"
    } else {
        "ENABLED"
    };
    md_line(
        out,
        "Hardware Acceleration",
        &format!(
            "saved {} / runtime {}",
            yn(d.gfx_hardware_acceleration),
            yn(d.runtime_hw_accel_enabled)
        ),
    );
    md_line(
        out,
        "Force DMA-BUF",
        &format!("saved {} / runtime {}", yn(d.dev_force_dmabuf), dmabuf),
    );
    md_line(
        out,
        "Force X11",
        &format!(
            "saved {} / runtime {}",
            yn(d.gfx_force_x11),
            yn(d.runtime_force_x11_active)
        ),
    );
    md_line(
        out,
        "GSK Renderer",
        &format!(
            "saved {} / runtime {}",
            opt(&d.gfx_gsk_renderer),
            opt(&d.env_gsk_renderer)
        ),
    );
    {
        let (renderer_runtime, renderer_adapters) = crate::renderer_decision_summary();
        md_line(
            out,
            "Renderer (Slint)",
            &format!(
                "saved {} / runtime {}",
                crate::ui_prefs::load().renderer,
                renderer_runtime
            ),
        );
        md_line(out, "GPU Adapters", &renderer_adapters);
        md_line(
            out,
            "UI Loop Latency",
            &format!(
                "{} ms (worst {} ms{})",
                crate::ui_watchdog::last_latency_ms(),
                crate::ui_watchdog::worst_latency_ms(),
                if crate::ui_watchdog::flagged() {
                    ", sustained degradation flagged"
                } else {
                    ""
                }
            ),
        );
    }
    md_line(out, "GDK Scale", &opt(&d.gfx_gdk_scale));
    md_line(out, "GDK DPI Scale", &opt(&d.gfx_gdk_dpi_scale));
    md_line(out, "Compositing Mode", compositing);
    md_line(
        out,
        "GPU",
        if d.runtime_gpu_name.is_empty() {
            "Unknown"
        } else {
            &d.runtime_gpu_name
        },
    );
    md_line(
        out,
        "GPU: NVIDIA",
        if d.runtime_has_nvidia { "Detected" } else { "No" },
    );
    md_line(
        out,
        "GPU: Intel",
        if d.runtime_has_intel { "Detected" } else { "No" },
    );
    md_line(
        out,
        "GPU: AMD",
        if d.runtime_has_amd { "Detected" } else { "No" },
    );
    md_line(
        out,
        "Desktop Environment",
        if d.runtime_desktop_environment.is_empty() {
            "Unknown"
        } else {
            &d.runtime_desktop_environment
        },
    );
    md_line(
        out,
        "Wayland",
        if d.runtime_is_wayland { "Yes" } else { "No (X11)" },
    );
    md_line(out, "VM", if d.runtime_is_vm { "Yes" } else { "No" });
    md_line(out, "Using Fallback", yn(d.runtime_using_fallback));
}

use qbz_app::diagnostics::RuntimeDiagnostics;

use super::helpers::{match_status, opt, row, yn};

pub(crate) fn build_graphics_rows(d: &RuntimeDiagnostics) -> Vec<crate::DiagRow> {
    // Active Slint renderer decision (wgpu / GL / software + why), recorded at
    // startup by select_slint_backend. Saved side = the Settings>Appearance
    // preference; runtime side = what actually got selected.
    let (renderer_runtime, renderer_adapters) = crate::renderer_decision_summary();
    let renderer_saved = crate::ui_prefs::load().renderer;
    let hw_saved = yn(d.gfx_hardware_acceleration);
    let hw_runtime = yn(d.runtime_hw_accel_enabled);
    let x11_saved = yn(d.gfx_force_x11);
    let x11_runtime = yn(d.runtime_force_x11_active);
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
    let gsk_saved = opt(&d.gfx_gsk_renderer);
    let gsk_runtime = opt(&d.env_gsk_renderer);
    let dmabuf_status = if d.dev_force_dmabuf == (dmabuf == "ENABLED") { 1 } else { 2 };

    // Event-loop responsiveness probe (#555): dispatch latency of a
    // cross-thread closure — renderer-independent, so a bad number here with
    // a healthy GPU points ABOVE the renderer. status 2 = sustained
    // degradation was flagged this session.
    let ui_latency = {
        let last = crate::ui_watchdog::last_latency_ms();
        let worst = crate::ui_watchdog::worst_latency_ms();
        if last == 0 && worst == 0 {
            "not sampled yet".to_string()
        } else {
            format!("{last} ms (worst {worst} ms)")
        }
    };
    let ui_latency_status = if crate::ui_watchdog::flagged() { 2 } else { 0 };

    let mut rows = vec![
        row("Renderer (Slint)", &renderer_saved, &renderer_runtime, 0),
        row("GPU Adapters", "—", &renderer_adapters, 0),
        row("UI Loop Latency", "—", &ui_latency, ui_latency_status),
        row(
            "Hardware Acceleration",
            hw_saved,
            hw_runtime,
            match_status(hw_saved, hw_runtime),
        ),
        row("Force DMA-BUF", yn(d.dev_force_dmabuf), dmabuf, dmabuf_status),
        row(
            "Force X11",
            x11_saved,
            x11_runtime,
            match_status(x11_saved, x11_runtime),
        ),
        row(
            "GSK Renderer",
            &gsk_saved,
            &gsk_runtime,
            match_status(&gsk_saved, &gsk_runtime),
        ),
        row("GDK Scale", &opt(&d.gfx_gdk_scale), "—", 0),
        row("GDK DPI Scale", &opt(&d.gfx_gdk_dpi_scale), "—", 0),
        row("Compositing Mode", "—", compositing, 0),
    ];
    rows.extend(gpu_env_rows(d));
    rows
}

/// The GPU-identity + desktop-environment tail of the Graphics rows — split
/// out purely to keep `build_graphics_rows` under the file's line budget.
fn gpu_env_rows(d: &RuntimeDiagnostics) -> Vec<crate::DiagRow> {
    vec![
        row(
            "GPU",
            "—",
            if d.runtime_gpu_name.is_empty() { "Unknown" } else { &d.runtime_gpu_name },
            0,
        ),
        row(
            "GPU: NVIDIA",
            "—",
            if d.runtime_has_nvidia { "Detected" } else { "No" },
            0,
        ),
        row(
            "GPU: Intel",
            "—",
            if d.runtime_has_intel { "Detected" } else { "No" },
            0,
        ),
        row(
            "GPU: AMD",
            "—",
            if d.runtime_has_amd { "Detected" } else { "No" },
            0,
        ),
        row(
            "Desktop Environment",
            "—",
            if d.runtime_desktop_environment.is_empty() {
                "Unknown"
            } else {
                &d.runtime_desktop_environment
            },
            0,
        ),
        row(
            "Wayland",
            "—",
            if d.runtime_is_wayland { "Yes" } else { "No (X11)" },
            0,
        ),
        row("VM", "—", if d.runtime_is_vm { "Yes" } else { "No" }, 0),
        row(
            "Using Fallback",
            "—",
            yn(d.runtime_using_fallback),
            if d.runtime_using_fallback { 2 } else { 0 },
        ),
    ]
}

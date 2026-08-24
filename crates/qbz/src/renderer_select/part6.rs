// SPLIT-EXCEPTION (crates/qbz/src/main.rs refactor): this file holds one
// tightly-sequential Rust function whose internal ordering/control-flow and
// captured-closure state make it unsafe to decompose further without a
// compiler in the loop (no `cargo check` is permitted for this refactor —
// see refactor-plans/crates__qbz__src__main.rs.md). Left whole, over the
// 130-line rule, as the documented rare exception it allows for.
use crate::*;

/// Decide + activate the Slint backend, returning whether the GPU (wgpu) renderer was
/// selected. `false` => femtovg-GL or the pure software renderer is active and the caller
/// must skip the wgpu shader underlay (neither exposes a WGPU28 GraphicsAPI). See the big
/// comment at the call site for the full rationale.
pub(crate) fn select_slint_backend() -> Result<bool, slint::PlatformError> {
    let (mut tier, mut source) = requested_renderer_tier();

    // macOS renders via Skia (see Cargo.toml) — femtovg-GL is not compiled there,
    // so the GL middle tier degrades to the wgpu path.
    if cfg!(target_os = "macos") && tier == RendererTier::FemtovgGl {
        log::info!("[renderer] GL tier unavailable on macOS -> using wgpu (skia)");
        tier = RendererTier::Wgpu;
        source.push_str(" (GL unavailable on macOS)");
    }

    // Make ONLY the miniplayer window borderless at CREATION (the flag is true solely
    // while MiniPlayerWindow::new() runs). Decorations cannot be reliably removed
    // post-creation on Wayland/KDE (server-side decorations are negotiated when the
    // surface is created), so the AppWindow keeps its system titlebar while the mini
    // never has one.
    let attributes_hook = |attributes: i_slint_backend_winit::winit::window::WindowAttributes| {
        // The miniplayer window is gone; the main window keeps its system chrome.
        let creating_mini = false;
        // Wayland app_id / X11 WM_CLASS: without an explicit name winit sends
        // no xdg_toplevel.set_app_id at all (and derives WM_CLASS from the
        // binary name), so the compositor cannot match the window to
        // com.blitzfc.qbz.desktop — blank dock icon, no running indicator,
        // and clicking the pin spawns a second instance (#544). Set on BOTH
        // windows so the miniplayer groups under the same icon.
        #[cfg(all(unix, not(target_os = "macos")))]
        let attributes = {
            use i_slint_backend_winit::winit::platform::wayland::WindowAttributesExtWayland;
            use i_slint_backend_winit::winit::platform::x11::WindowAttributesExtX11;
            // Both traits expose `with_name`; UFCS picks each apart.
            let attributes = WindowAttributesExtWayland::with_name(
                attributes,
                "com.blitzfc.qbz",
                "com.blitzfc.qbz",
            );
            WindowAttributesExtX11::with_name(attributes, "com.blitzfc.qbz", "com.blitzfc.qbz")
        };
        // Per-window swapchain alpha (vendored femtovg-wgpu patch): this hook runs
        // on the event loop thread right before the window ADAPTER — and therefore
        // its renderer backend — is created, and the backend CAPTURES the flag at
        // construction (surface (re)creation happens later and repeats on every
        // Wayland re-show, so a live read there would leak this latched value
        // across windows). Net effect: only the miniplayer gets a transparent
        // (blended) swapchain, for its whole lifetime; the main window keeps an
        // Opaque one, sparing the compositor a full-window alpha blend every frame.
        #[cfg(not(target_os = "macos"))]
        i_slint_renderer_femtovg::wgpu::set_surface_prefers_transparent(creating_mini);
        // macOS custom chrome (owner decision 2026-07-03, default ON there):
        // keep the native decorations but make the title bar transparent and
        // extend the content underneath — the native traffic lights float over
        // the app's own header (which reserves a left inset for them). This is
        // the macOS analog of Linux's `no-frame`; we never draw Mac controls.
        // Same restart-to-apply semantics: attributes are fixed at creation.
        #[cfg(target_os = "macos")]
        let attributes = if !creating_mini && !crate::ui_prefs::load().use_system_title_bar {
            use i_slint_backend_winit::winit::platform::macos::WindowAttributesExtMacOS;
            attributes
                .with_titlebar_transparent(true)
                .with_title_hidden(true)
                .with_fullsize_content_view(true)
        } else {
            attributes
        };
        if creating_mini {
            attributes.with_decorations(false)
        } else {
            attributes
        }
    };

    match tier {
        RendererTier::Wgpu => {
            // Explicit configuration instead of `default()`: default leaves the
            // adapter PowerPreference at None. Prefer the low-power (integrated)
            // adapter — this UI is mostly idle — EXCEPT on hybrid desktops,
            // where the iGPU can't present the surface (#542, see
            // default_wgpu_power_preference). WGPU_POWER_PREF still wins if
            // set (same as WGPUSettings::default()).
            let mut wgpu_settings = slint::wgpu_28::WGPUSettings::default();
            // Resolution order: WGPU_POWER_PREF env (debug) > persisted
            // "Preferred GPU" setting (Settings > Renderer) > auto default.
            wgpu_settings.power_preference = slint::wgpu_28::wgpu::PowerPreference::from_env()
                .or_else(gpu_power_from_prefs)
                .unwrap_or_else(default_wgpu_power_preference);
            // Alternate-adapter rung: the previous start died on the adapter
            // this preference picks, so flip it (even over WGPU_POWER_PREF —
            // anyone setting that env is debugging and reads the log line).
            if WGPU_ALT_ADAPTER.load(std::sync::atomic::Ordering::Relaxed) {
                use slint::wgpu_28::wgpu::PowerPreference;
                wgpu_settings.power_preference = match wgpu_settings.power_preference {
                    PowerPreference::HighPerformance => PowerPreference::LowPower,
                    _ => PowerPreference::HighPerformance,
                };
                log::warn!(
                    "[renderer] alternate-adapter rung: flipped power preference to {:?}",
                    wgpu_settings.power_preference
                );
            }
            log::info!(
                "[renderer] selecting wgpu (GPU) renderer (power_preference={:?})",
                wgpu_settings.power_preference
            );
            // Tray-restore SEGFAULT fix (2026-07-18): on Wayland, hide()
            // destroys the winit window and show() re-runs the femtovg-wgpu
            // surface setup; with `Automatic` that spins up a fresh VkInstance
            // + VkDevice EVERY restore and the NVIDIA driver segfaults on the
            // churn (null call in libnvidia-glcore inside `set_surface`;
            // close-to-tray → restore). On Linux+Wayland create the wgpu stack
            // ONCE here and hand it over as `Manual`, so a restore only
            // re-creates the surface/swapchain on the same device. Other
            // platforms don't destroy on hide — keep `Automatic` there, and
            // fall back to it if the shared stack cannot be created.
            let shared_stack = if cfg!(target_os = "linux")
                && std::env::var_os("WAYLAND_DISPLAY").is_some()
            {
                create_shared_wgpu_stack(&wgpu_settings)
            } else {
                None
            };
            match shared_stack {
                Some((instance, adapter, device, queue)) => {
                    slint::BackendSelector::new()
                        .require_wgpu_28(slint::wgpu_28::WGPUConfiguration::Manual {
                            instance,
                            adapter,
                            device,
                            queue,
                        })
                        .with_winit_window_attributes_hook(attributes_hook)
                        .select()?;
                }
                None => {
                    slint::BackendSelector::new()
                        .require_wgpu_28(slint::wgpu_28::WGPUConfiguration::Automatic(
                            wgpu_settings,
                        ))
                        .with_winit_window_attributes_hook(attributes_hook)
                        .select()?;
                }
            }
        }
        RendererTier::FemtovgGl => {
            log::info!(
                "[renderer] selecting femtovg GL renderer (winit + renderer-femtovg); \
                 shader underlay disabled"
            );
            slint::BackendSelector::new()
                .backend_name("winit".to_string())
                .renderer_name("femtovg".to_string())
                .with_winit_window_attributes_hook(attributes_hook)
                .select()?;
        }
        RendererTier::Software => {
            log::info!(
                "[renderer] selecting software renderer (winit + renderer-software); \
                 shader underlay disabled"
            );
            slint::BackendSelector::new()
                .backend_name("winit".to_string())
                .renderer_name("software".to_string())
                .with_winit_window_attributes_hook(attributes_hook)
                .select()?;
        }
    }
    let _ = RENDERER_DECISION.set(RendererDecision {
        tier: match tier {
            RendererTier::Wgpu if cfg!(target_os = "macos") => "skia (Metal)",
            RendererTier::Wgpu => "wgpu (femtovg)",
            RendererTier::FemtovgGl => "GL (femtovg)",
            RendererTier::Software => "software",
        },
        source,
    });
    Ok(tier == RendererTier::Wgpu)
}


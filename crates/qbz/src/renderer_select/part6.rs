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
    // never has one. See `window_attributes_hook` (window_attributes_hook.rs).
    let attributes_hook = window_attributes_hook;

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


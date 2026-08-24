//! QBZ Slint MVP binary.
//!
//! A native Slint front end for QBZ built on the framework-agnostic
//! `qbz-app` / `qbz-core` stack — no Tauri, no WebView. See the MVP ADR
//! (`qbz-nix-docs/qbz-adr/qbz_slint_functional_poc_adr.md`).
//!
//! Lives only on the private `slint-mvp` branch (ADR-007). The Slint UI
//! tree is compiled in the `qbz-ui` library crate (its `build.rs` runs
//! `slint-build` on `ui/app.slint`); this binary re-exports all generated
//! types at the crate root so existing `crate::AppWindow` / `crate::*State`
//! references resolve unchanged.
//!
//! Status: foundation tokens, login screen, app shell, functional
//! system-browser OAuth, saved-session restore, and a real Discover /
//! Home view fed by the Qobuz discover index with cached artwork.

pub use qbz_ui::*;

mod about;
mod adapter;
mod album;
mod album_map;
mod artist;
mod artist_blacklist;
mod artist_prefs;
mod artist_releases;
mod artwork;
mod auth;
mod auto_theme;
mod blacklist_manager;
mod booklet;
mod commands;
mod custom_artwork;
mod custom_theme;
mod device_cap;
mod diagnostics;
#[cfg(target_os = "linux")]
mod glibc_compat;
mod log_viewer;
mod sleep_timer;
pub use qbz_text_utils::{dates, strip_html};
mod deep_link;
mod discover_browse;
mod discover_prefs;
mod discovery_dismiss;
mod fav_cache;
mod favorites;
mod favorites_prefs;
mod external_reco;
mod foryou;
mod genre_filter;
use qbz_dac_wizard as dac_wizard;
mod home;
mod immersive;
mod info_modals;
mod keybindings;
mod label;
mod library_all;
mod library_by_artist;
mod library_by_label;
mod link_resolver;
mod location_view;
mod mix;
mod musician;
mod myqbz;
mod myqbz_add;
mod myqbz_cover;
mod myqbz_detail;
mod myqbz_edit;
mod myqbz_mix;
mod myqbz_play;
mod myqbz_prefs;
mod myqbz_view_prefs;
mod nav;
mod pinned;
mod pinned_section;
mod play_history;
mod playback;
mod queue;
mod remote_stream;
mod drag;
mod ephemeral;
mod folders;
mod library_db;
mod local_favorites;
mod local_library;
mod local_playlist;
mod local_library_settings;
#[cfg(target_os = "macos")]
mod macos_chrome;
mod media_controls;
mod locallibrary_prefs;
mod tag_editor;
mod offline;
mod offline_cache;
mod offline_favorites;
mod visualizer;
mod offline_manager;
mod offline_mode;
mod playlist;
mod playlist_browse;
mod playlist_import;
mod playlist_manager;
mod playlist_snapshot;
mod playlist_suggestions;
mod playlist_suggestions_dismiss;
mod playlist_picker;
mod playlist_picker_apply;
mod playlist_picker_load;
mod playlist_membership;
mod playlist_membership_qobuz;
mod quality;
mod reco;
mod reco_dismiss;
mod recently;
mod scrobble;
mod scrobbler_settings;
mod search;
mod selection;
mod session_persist;
mod search_service;
mod single_instance;
// WGPU UNDERLAY SPIKE: GPU fragment-shader background for ImmersiveView.
mod shader_underlay;
mod settings;
mod share;
mod sidebar;
mod startup_defer;
mod suggestions;
mod theme;
pub use qbz_slint_common::toast;
mod tray;
mod tray_settings;
mod ui_prefs;
mod viewport;
mod ui_watchdog;
mod whats_new;

// --- main.rs split (crates/qbz/src/main.rs refactor) -----------------------
// The pre-`fn main()` free-function section (formerly ~7,250 lines) is split
// into these sibling clusters, in original top-to-bottom order. Each is
// glob-re-exported here so the crate-root flat namespace (and `fn main()`'s
// own body, still below) resolves every name exactly as before the split.
// See refactor-plans/crates__qbz__src__main.rs.md.
mod shell_bootstrap;
mod nav_flags_chrome;
mod row_toggles;
mod navigate_album_artist;
mod playlist_picker_helpers;
mod navigate_search;
mod navigate_recent_library;
mod drag_sidebar;
mod folder_editor;
mod wire_playlist_manager;
mod wire_myqbz;
mod renderer_select;

pub(crate) use shell_bootstrap::*;
pub(crate) use nav_flags_chrome::*;
pub(crate) use row_toggles::*;
pub(crate) use navigate_album_artist::*;
pub(crate) use playlist_picker_helpers::*;
pub(crate) use navigate_search::*;
pub(crate) use navigate_recent_library::*;
pub(crate) use drag_sidebar::*;
pub(crate) use folder_editor::*;
pub(crate) use wire_playlist_manager::*;
pub(crate) use wire_myqbz::*;
pub(crate) use renderer_select::*;

use std::sync::Arc;

use i_slint_backend_winit::{
    winit::event::{ElementState, MouseButton, TouchPhase, WindowEvent},
    winit::keyboard::{Key, NamedKey},
    EventResult, WinitWindowAccessor,
};
use slint::Model;

use adapter::SlintAdapter;
use commands::AppCommand;
use qbz_app::shell::AppRuntime;

/// Login Terms-of-Service link target.
const QOBUZ_TOS_URL: &str = "https://www.qobuz.com/us-en/legal/terms";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // UI SCALE PRESET — must run FIRST: SLINT_SCALE_FACTOR has to be in the
    // environment before the backend/window exist (winit reads it at window
    // creation), and `set_var` must run before any thread spawns (the tokio
    // runtime below). The env var OVERRIDES the compositor DPR rather than
    // multiplying it, so the last observed real DPR is baked into the value
    // (see `last_dpr` in ui_prefs). Default preset => the var is NOT set at
    // all: stock DPR handling (incl. live monitor changes) stays intact.
    let ui_scale_factor = {
        let prefs = crate::ui_prefs::load();
        let factor = crate::ui_prefs::ui_scale_factor(&prefs.ui_scale);
        if factor != 1.0 {
            let effective = prefs.last_dpr.max(0.5) * factor;
            std::env::set_var("SLINT_SCALE_FACTOR", effective.to_string());
        }
        let _ = ACTIVE_UI_SCALE.set(factor);
        factor
    };

    // Composite logger: stderr (unchanged) + a bounded in-memory ring + an on-disk
    // file, all redacted at the write choke point. Feeds the in-app log viewer and
    // the diagnostics bundle. Honours RUST_LOG (default "info").
    qbz_log::install("info");
    if ui_scale_factor != 1.0 {
        log::info!(
            "[ui-scale] preset factor {ui_scale_factor} -> SLINT_SCALE_FACTOR={}",
            std::env::var("SLINT_SCALE_FACTOR").unwrap_or_default()
        );
    }

    // XDG deep link (Exec=qbz %u): capture a Qobuz link from argv BEFORE the
    // single-instance guard below — a second launch must read its own argv
    // before deciding to raise the primary and exit (the guard forwards the
    // stashed URL over D-Bus; as primary it drains at the end of enter_shell).
    deep_link::capture_argv();
    // Artwork decode targets grow with the preset so covers stay sharp at
    // Large/XL (and shrink RAM at Small). Set before any artwork job runs.
    crate::artwork::set_ui_scale_factor(ui_scale_factor);

    let tokio_rt = tokio::runtime::Runtime::new()?;
    let _enter = tokio_rt.enter();

    // Single-instance guard (issue #559, Tauri parity): a second launch asks
    // the running instance to raise its window (MPRIS Raise → the tray raise
    // path) and exits instead of starting a duplicate player. Any D-Bus
    // problem falls through as primary — the guard never blocks startup.
    #[cfg(target_os = "linux")]
    if !single_instance::acquire_or_raise() {
        log::info!("[qbz-slint] another instance owns the session bus name — raised it, exiting");
        return Ok(());
    }

    // STARTUP CRASH-CHAIN WATCHDOG — armed after the single-instance guard
    // (a raise-and-exit must not count as a crash) and BEFORE any risky init
    // (renderer probing, window creation, view restore). Cleared by the same
    // liveness proof that disarms the renderer sentinel. See
    // `arm_startup_probe` for the recovery ladder.
    arm_startup_probe();

    // RENDERER SELECTION — pick wgpu (GPU) vs femtovg GL vs Slint's software
    // renderer BEFORE the first window is created. All three use the winit backend,
    // so the tray/miniplayer WinitWindowAccessor stays valid either way.
    //
    // Why this is not an unconditional `require_wgpu_28` anymore: on a host with a
    // real GPU (dev boxes, Apple Silicon) wgpu flies. But on a host WITHOUT one — a
    // VM with no GPU passthrough — wgpu happily binds a *software* Vulkan/GL adapter
    // (llvmpipe / lavapipe) and then CPU-rasterizes the entire UI every frame at
    // 60fps, which pegs the CPU and makes the app crawl. The old Tauri/WebKitGTK
    // build never hit this because its software path is far more optimized. And on
    // weak-GPU hosts (Raspberry Pi-class) the real Vulkan adapter (v3dv/panfrost)
    // is itself the slow path — Mesa's GLES driver is the fast one there, so those
    // get the femtovg GL middle tier. Everything without a usable GPU falls back
    // to Slint's pure software renderer. The wgpu shader underlay is non-fatal (it
    // just stays dark), so GL/software modes only lose the immersive visualizer
    // eye-candy — the rest of the UI is intact and fast.
    //
    // Manual override: QBZ_RENDERER=software|cpu|soft forces software;
    // QBZ_RENDERER=gl|gles|femtovg forces femtovg GL; QBZ_RENDERER=gpu|wgpu|hardware|hw
    // forces wgpu; unset / "auto" auto-detects.
    let use_gpu_renderer = select_slint_backend()?;

    // Wayland/X11 app identity that SURVIVES surface recreation: the winit
    // attributes hook (select_slint_backend) only runs once per window
    // adapter, and Slint's Wayland hide() destroys the toplevel and rebuilds
    // it from DEFAULT attributes (suspend(), i-slint-backend-winit).
    // ensure_window() re-applies the context xdg_app_id on EVERY creation, so
    // this is the only way a re-shown window keeps grouping under
    // com.blitzfc.qbz.desktop (#618). Must run after the backend is selected
    // (needs the global context) and before any window is created.
    if let Err(e) = slint::set_xdg_app_id("com.blitzfc.qbz") {
        log::warn!("[qbz-slint] set_xdg_app_id failed: {e}");
    }

    // UI language: resolve the persisted language BEFORE the first window is
    // created, and set the Rust-side language now so `t()`/date helpers are
    // correct from the first call. The persisted key may be "auto" (follow the
    // OS locale) — resolve that to a concrete language. set_language() drives
    // our Rust-side `t`/dates helpers.
    //
    // NOTE: select_bundled_translation() operates on the component's GLOBAL
    // translation context, which only exists AFTER AppWindow::new() — calling
    // it before that is a no-op (returns NoTranslationsBundled) and leaves the
    // first paint in English. So we compute `lang` here, set the Rust language,
    // and defer the Slint translation switch + label reseed to just after
    // AppWindow::new() below.
    let lang = {
        let persisted = crate::ui_prefs::load().language;
        let lang = if persisted == "auto" {
            qbz_i18n::resolve_auto()
        } else {
            qbz_i18n::set_language(&persisted);
            qbz_i18n::current_language()
        };
        qbz_i18n::set_language(lang);
        lang
    };

    let window = AppWindow::new()?;
    // Publish the window to the single-instance Present() handler (exported
    // since acquire_or_raise above) — a second launch can now raise even the
    // login screen, no MPRIS needed. Also drains a Present that raced in
    // between the guard and this point.
    #[cfg(target_os = "linux")]
    single_instance::bind_window(window.as_weak());
    // Renderer auto-revert sentinel: disarmed on the FIRST real user input
    // (or window close request) via the winit event filter — proof the app
    // reached a usable state — with a 30s fallback for no-touch sessions.
    // The old fixed 5s disarm lost the race against late startup crashes
    // (#558: the swapchain error lands seconds after first paint, the timer
    // had already disarmed, and the crash looped forever).
    let _renderer_sentinel_timer = slint::Timer::default();
    _renderer_sentinel_timer.start(
        slint::TimerMode::SingleShot,
        std::time::Duration::from_secs(30),
        || {
            disarm_renderer_sentinel_on_liveness("30s fallback");
        },
    );
    // Event-loop responsiveness watchdog (#555): background probe thread,
    // read by the Diagnostics panel. Detection only — never switches tiers.
    ui_watchdog::spawn();
    // Persist the REAL compositor DPR as `last_dpr` once the surface is
    // mapped (winit reports the true value there — unlike right after
    // creation on Wayland). Read from the WINIT window, not the Slint one:
    // SLINT_SCALE_FACTOR overrides Slint's factor but winit still sees the
    // compositor's. The next scaled launch bakes this into the env value.
    let _dpr_probe_timer = slint::Timer::default();
    let sentinel_weak = window.as_weak();
    _dpr_probe_timer.start(
        slint::TimerMode::SingleShot,
        std::time::Duration::from_secs(5),
        move || {
            if let Some(w) = sentinel_weak.upgrade() {
                w.window().with_winit_window(|win| {
                    let real_dpr = win.scale_factor() as f32;
                    let mut prefs = crate::ui_prefs::load();
                    if real_dpr > 0.1 && (prefs.last_dpr - real_dpr).abs() > 0.01 {
                        log::info!("[ui-scale] observed compositor DPR {real_dpr} -> persisted");
                        prefs.last_dpr = real_dpr;
                        crate::ui_prefs::save(&prefs);
                    }
                });
            }
        },
    );
    // Now that the AppWindow (and its translation global context) exists, switch
    // the Slint bundled translations to `lang` and reseed the non-reactive
    // option arrays so the first paint is fully in the persisted language.
    if let Err(e) = slint::select_bundled_translation(lang) {
        log::warn!("[qbz-slint] select_bundled_translation('{lang}') failed: {e:?}");
    }
    reseed_i18n_labels(&window);
    // Shader scenes need the wgpu underlay: on the femtovg-GL / software tiers
    // they would render black, so hide them from the immersive picker + `g`.
    window
        .global::<ImmersiveState>()
        .set_shader_scenes_available(use_gpu_renderer);
    // App-wide dynamic background ("modo Cider"): available only on the wgpu
    // tier (same reason as the shader scenes — it would render black on
    // GL/software where reduce-motion is forced on). Gates the whole picker row.
    {
        let ap = window.global::<AppearanceState>();
        ap.set_app_background_available(use_gpu_renderer);
        // Live-tunable look knobs (QBZ_BG_DIM / QBZ_BG_SURFACE_ALPHA in [0,1])
        // so the appearance can be dialed in ONE smoke session without a rebuild,
        // then baked to the defaults in state.slint.
        if let Some(f) = std::env::var("QBZ_BG_DIM")
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
        {
            ap.set_app_background_dim(f.clamp(0.0, 1.0));
        }
        if let Some(f) = std::env::var("QBZ_BG_SURFACE_ALPHA")
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
        {
            ap.set_app_background_surface_alpha(f.clamp(0.0, 1.0));
        }
        if let Some(f) = std::env::var("QBZ_BG_BAR_ALPHA")
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
        {
            ap.set_app_background_bar_alpha(f.clamp(0.0, 1.0));
        }
    }
    // Weak renderer tiers: every animation frame is a full-window femtovg
    // repaint — step loading indicators / eq bars at ~8fps (coarse clock in
    // AppShell) instead of display rate.
    window
        .global::<ShellState>()
        .set_reduce_motion(!use_gpu_renderer);
    // Interface-size preset: publish the factor so `.slint` bindings that must
    // stay physically constant (the window minimums) can divide it back out.
    // Extra small also gets the font compensation: a plain 0.8 drops body text
    // to 12px; boosting Typography tokens ~10% lands at ~13px (readable) while
    // layout metrics keep the full 0.8 — that density is the point of XS.
    // Small (0.9) needs none: body lands at 13.5px on its own.
    window.global::<UiScale>().set_factor(ui_scale_factor);
    if ui_scale_factor < 0.85 {
        window.global::<Typography>().set_boost(1.1);
    }
    // Diagnostic for the circle-AA investigation (and the future UI-scale
    // presets): femtovg's fringe AA halves at fractional scale factors
    // (internal dpi = ceil(scale)), so knowing the real factor matters.
    log::info!(
        "[renderer] window scale factor: {}",
        window.window().scale_factor()
    );
    // Tell the user their renderer override was rolled back (set in
    // renderer_tier_from_prefs when the previous start died before painting).
    if RENDERER_REVERTED.load(std::sync::atomic::Ordering::Relaxed) {
        crate::toast::warning(
            &window,
            qbz_i18n::t("Renderer setting reverted to Auto — the previous start didn't finish"),
        );
    }
    // Same idea for the auto-detect path (#542): wgpu crashed pre-paint last
    // time, this start persisted the GL fallback instead.
    if RENDERER_DEGRADED.load(std::sync::atomic::Ordering::Relaxed) {
        // Generic on purpose: the ladder can land on GL or software.
        crate::toast::warning(
            &window,
            qbz_i18n::t(
                "Renderer switched automatically — the previous renderer failed to start",
            ),
        );
    }
    install_browser_mouse_nav(&window);
    wire_window_controls(&window);
    // FONT TEST (slint-mvp): render with bundled Inter 18pt. Inter is a
    // clean, screen-tuned UI face; combined with the femtovg #5177/#11335
    // text fixes this is the candidate for the final look. Flip
    // `FONT_TEST_INTER` to false to fall back to the KDE system font.
    const FONT_TEST_INTER: bool = true;
    if FONT_TEST_INTER {
        log::info!("[qbz-slint] font test: using bundled Inter 18pt");
        window.set_system_font("Inter 18pt".into());
    } else if let Some(font) = system_font_family() {
        log::info!("[qbz-slint] using system font: {font}");
        window.set_system_font(font.into());
    }

    // Restore the persisted shell chrome before the first paint. The sidebar
    // defaults open (state 0), which is what mounts the Large cover + spectrum
    // dock; a closed sidebar simply leaves `large-active` false.
    let restored_prefs = crate::ui_prefs::load();
    {
        let shell = window.global::<ShellState>();
        // Restore the persisted sidebar state (0 open / 1 mini / 2 closed) +
        // section-nav placement before the shell renders.
        shell.set_sidebar_state(restored_prefs.sidebar_state);
        shell.set_nav_in_sidebar(restored_prefs.nav_in_sidebar);
        shell.set_nav_header_compact(restored_prefs.nav_header_compact);
        // Large dock toggles — restore the persisted visualizer state + spectrum
        // choice (default ON / Bars) before the shell renders.
        shell.set_large_visualizer_on(restored_prefs.large_visualizer);
        shell.set_large_spectrum_mode(crate::ui_prefs::large_spectrum_mode_index(
            &restored_prefs.large_spectrum_mode,
        ));
    }
    // Main window geometry: restore the persisted size/position/maximized
    // state. Shared with the tray/miniplayer re-show paths (#618) — the
    // clamping and plausibility rules live in the helper's doc.
    restore_main_window_geometry(&window);
    // NOTE: the FFT tap is primed AFTER visualizer::install() (further below) — not
    // here — because install() registers the `on_set_enabled` handler this call
    // depends on.
    // Startup audit 2026-08-20: load persisted UI prefs ONCE for this whole
    // boot-seed sequence. Below this point, through the theme restore (all
    // still before window.show()), prefs are read repeatedly but never
    // written — this used to be ~15 separate disk reads + JSON parses of
    // the same file. Event handlers further down still call
    // `ui_prefs::load()` fresh at click-time; only this synchronous,
    // pre-paint seeding reuses one snapshot.
    let boot_prefs = crate::ui_prefs::load();
    window
        .global::<AppearanceState>()
        .set_album_header_gradient(boot_prefs.album_header_gradient);
    window
        .global::<AppearanceState>()
        .set_intelligent_search(boot_prefs.intelligent_search);
    // Appearance toggles that used to be live-only (no persistence): seed the
    // live globals from the persisted prefs so the user's choice survives a
    // restart. Their Rust handlers now persist via on_appearance_bool/select.
    {
        let prefs = boot_prefs.clone();
        let appearance = window.global::<AppearanceState>();
        appearance.set_window_title_show(prefs.window_title_show);
        appearance.set_show_volume_steppers(prefs.show_volume_steppers);
        appearance.set_sidebar_playlist_collage(prefs.sidebar_playlist_collage);
        appearance.set_local_library_track_artwork(prefs.local_library_track_artwork);
        appearance.set_in_app_toasts(prefs.in_app_toasts);
        appearance.set_theme_filter(prefs.theme_filter);
        window
            .global::<SidebarState>()
            .set_playlist_collage(prefs.sidebar_playlist_collage);
        window
            .global::<ToastState>()
            .set_enabled(prefs.in_app_toasts);
    }
    // Custom window chrome — seeded before the first `show()`, since
    // `AppWindow.no-frame` reads `use-system-title-bar` at surface creation
    // (decorations negotiate then on Wayland), and the macOS attributes hook
    // reads the same pref straight from ui_prefs.
    {
        let prefs = boot_prefs.clone();
        let appearance = window.global::<AppearanceState>();
        appearance.set_use_system_title_bar(prefs.use_system_title_bar);
        // Applied chrome state — what this window is actually created with;
        // the settings toggle edits the pref above. On Linux the
        // appearance-bool handler mirrors pref -> active live; on macOS it
        // does not (overlay attributes are fixed at creation), so this seed
        // is the value for the whole session there.
        appearance.set_system_title_bar_active(prefs.use_system_title_bar);
        appearance.set_hide_title_bar(prefs.hide_title_bar);
        appearance.set_show_window_controls(prefs.show_window_controls);
        appearance.set_wc_position_index(if prefs.wc_position == "left" { 0 } else { 1 });
    }
    // App-wide dynamic background mode (0 = off, 1 = ambient, 2 = blurred).
    window.global::<AppearanceState>().set_app_background_mode_index(
        crate::ui_prefs::app_background_index(&boot_prefs.app_background),
    );
    // System Notifications toggle: seed the UI + the poll-thread atomic gate.
    {
        let sys_notif = boot_prefs.system_notifications;
        window
            .global::<AppearanceState>()
            .set_system_notifications(sys_notif);
        playback::NOTIFICATIONS_ENABLED
            .store(sys_notif, std::sync::atomic::Ordering::Relaxed);
    }
    window.global::<AppearanceState>().set_startup_page_index(
        crate::ui_prefs::startup_page_index(&boot_prefs.startup_page),
    );
    // Language selector: seed the dropdown index from the persisted key (the raw
    // user choice, "auto" -> index 0). The live translation was already applied
    // before the window was created.
    window.global::<AppearanceState>().set_language_index(
        crate::ui_prefs::language_index(&boot_prefs.language),
    );

    // Theme: seed the dropdown list from the Rust registry, then restore the
    // persisted theme (slug is the source of truth; the dropdown index is
    // derived). Fresh profiles default to OLED Dark (owner decision). This must
    // run before the shell renders so the first paint is the right palette.
    {
        let appearance = window.global::<AppearanceState>();
        let prefs = boot_prefs.clone();
        // Seed the dropdown honoring the persisted list filter (All/Dark/Light)
        // so the list and the filter icon agree on the first paint.
        let filter = prefs.theme_filter;
        let labels: Vec<slint::SharedString> = crate::theme::filtered_dropdown_labels(filter)
            .into_iter()
            .map(slint::SharedString::from)
            .collect();
        appearance.set_themes(slint::ModelRc::new(slint::VecModel::from(labels)));

        let slug = prefs.theme;
        let is_auto = slug == crate::theme::AUTO_SLUG;
        let is_custom = slug == crate::theme::CUSTOM_SLUG;
        // Highlight the active theme within the (possibly narrowed) list; -1 when
        // it is filtered out — the palette is still applied below via the slug.
        let selected_index = crate::theme::filtered_selected_index_for_slug(&slug, filter);
        appearance.set_theme_index(selected_index);
        appearance.set_theme_is_auto(is_auto);
        appearance.set_theme_is_custom(is_custom);
        // Auto-theme controls read from the persisted source; seed them so they
        // reflect the saved choice when Settings opens.
        crate::auto_theme::seed_state(&window);
        // Custom-theme editor swatches read from custom_theme.json; seed them so
        // the editor reflects the saved base when Settings opens.
        crate::custom_theme::seed_state(&window);
        if is_auto {
            appearance.set_theme_is_system(false);
            // Generate + apply the dynamic palette (falls back to OLED on error).
            crate::auto_theme::apply_startup(&window);
        } else if is_custom {
            appearance.set_theme_is_system(false);
            // Derive + apply the persisted custom palette (seeds OLED if absent).
            crate::custom_theme::apply_startup(&window);
        } else {
            let id = crate::theme::id_for_slug(&slug);
            appearance.set_theme_is_system(id == qbz_theme::ThemeId::System);
            crate::theme::apply_theme(&window, id);
        }
        // Keep the legacy ThemeState.mode in sync with the UNFILTERED dropdown
        // index for any residual reads (the filtered index can be -1); the
        // palette itself is driven above.
        window
            .global::<ThemeState>()
            .set_mode(crate::theme::selected_index_for_slug(&slug));
    }

    // Tell the tray settings UI which platform it's on so it can show the
    // macOS-only controls ("Menu Bar" header, hide-Dock toggle) and hide the
    // Linux/Windows-only minimize-to-tray row.
    window
        .global::<AppearanceState>()
        .set_is_macos(cfg!(target_os = "macos"));


    let app_runtime = Arc::new(AppRuntime::with_visualizer(SlintAdapter::new(window.as_weak())));

    // ImmersiveView audio visualizers: spawn the frontend-agnostic FFT producer
    // against the runtime's tap and start the 30fps drain into VisualizerState.
    // Inert (tap disabled, no capture / no FFT cost) until the immersive view
    // opens. Must run on the UI thread before window.run().
    visualizer::install(&window, &app_runtime);

    // Prime the FFT tap if we restored straight into Large with the visualizer ON.
    // This MUST run AFTER visualizer::install() — install() registers the
    // `on_set_enabled` handler, so an earlier invoke would no-op (the cause of the
    // dock spectrum sitting idle on the very first playback until the eye toggle
    // was pressed). The AppShell `changed viz-should-run` handler covers every
    // later transition; this only seeds the initial value.
    {
        let shell = window.global::<ShellState>();
        if shell.get_large_active() && shell.get_large_visualizer_on() {
            window.global::<VisualizerState>().invoke_set_enabled(true);
        }
    }

    // WGPU UNDERLAY: capture Slint's own wgpu Device/Queue at RenderingSetup
    // so shader_underlay allocates its texture + submits on the SAME device Slint
    // renders with (mandatory for Image::try_from). The render itself happens in
    // the 30fps drain (visualizer.rs). Only one rendering notifier is allowed per
    // window; the shader underlay owns it. Errors here are non-fatal — the shader
    // just stays dark and the rest of the UI is unaffected.
    //
    // Only registered when the GPU (wgpu) renderer was actually selected. In software
    // mode there is no WGPU28 GraphicsAPI to hook, so registering it would be a no-op
    // at best — skip it so software mode carries zero wgpu machinery.
    if use_gpu_renderer {
        if let Err(e) = window
            .window()
            .set_rendering_notifier(move |state, graphics_api| {
                match state {
                    slint::RenderingState::RenderingSetup => {
                        if let slint::GraphicsAPI::WGPU28 { device, queue, .. } = graphics_api {
                            crate::shader_underlay::setup(device.clone(), queue.clone());
                        }
                    }
                    slint::RenderingState::RenderingTeardown => {
                        crate::shader_underlay::teardown();
                        // The stale ImmersiveState.shader-texture (a wgpu texture
                        // from the surface being destroyed) is cleared in
                        // tray::hide_window BEFORE the surface teardown — NOT here:
                        // setting a property inside this notifier re-borrows the
                        // winit adapter's RefCell (panic on close-to-tray).
                    }
                    _ => {}
                }
            })
        {
            log::warn!("[shader] set_rendering_notifier failed: {e:?} — underlay disabled");
        }
    } else {
        log::info!("[shader] software renderer active — wgpu shader underlay disabled");
    }

    // MusicBrainz cache — opens a SQLite store at
    // <data-dir>/qbz/cache/musicbrainz_cache.db so artist metadata
    // and relationships persist across sessions (matches Tauri's
    // MusicBrainzCache init path). Failure to open just degrades to
    // direct network calls — the methods skip the cache when none
    // is set. Startup audit 2026-08-20: moved off the synchronous
    // startup path (was a blocking SQLite open before the first paint) —
    // see startup_defer::spawn_musicbrainz_cache.
    startup_defer::spawn_musicbrainz_cache(&tokio_rt, app_runtime.clone());

    // MusicBrainz opt-out seed — drive the core client's enabled flag from the
    // persisted UI pref (default ON). Without this the client stays hardcoded ON
    // and the Settings toggle can't turn it off. main() is sync, so spawn on the
    // runtime; this runs before the first artist / playlist-suggestions load.
    {
        let runtime = app_runtime.clone();
        tokio_rt.spawn(async move {
            let mb_on = crate::ui_prefs::load().musicbrainz_enabled;
            runtime.core().musicbrainz_set_enabled(mb_on).await;
        });
    }

    // Shared QBZ image cache for album artwork. Startup audit 2026-08-20:
    // the SQLite open (WAL pragma + CREATE TABLE) used to run here,
    // synchronously, before the first paint. `image_cache` is now the
    // eventual handle — an empty `Option` published immediately — and the
    // real open (+ eviction pass) happens on a background task; see
    // startup_defer::spawn_image_cache. Every closure below still just
    // clones this same `Arc`, so nothing downstream changes.
    let image_cache = startup_defer::spawn_image_cache(&tokio_rt);

    // Audio + Playback settings stores, opened once for the app lifetime.
    let settings_ctx = settings::SettingsCtx::open();

    // Offline-MODE engine: connectivity monitoring runs for the whole app
    // lifetime (login screen included — the restore flow and the D2 recovery
    // banner both depend on it). Per-user state binds later on activation.
    offline_mode::start();
    // Mirror engine status into the OfflineState Slint global (login
    // affordances + the D2 recovery banner) and seed has-previous-session.
    offline_mode::start_ui_forwarder(window.as_weak());

    // Offline EDGE reactions (D11/D12b). On online→offline: a user standing
    // on a placeholder-blocked Qobuz view auto-navigates to LocalLibrary (the
    // offline default view), the sidebar re-renders from cache (the offline
    // filter keeps locals + mixed-with-local-content, real names intact), and
    // an open My QBZ grid/detail reloads so unavailable items drop (D11.c).
    // On offline→online: NO navigation (blocked views unblock naturally);
    // the sidebar reloads the full Qobuz set.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        tokio_rt.spawn(async move {
            let mut rx = offline_mode::engine().subscribe();
            let initial = *rx.borrow_and_update();
            let mut was_offline = initial.is_offline();
            let mut was_conn_down =
                initial.connectivity == qbz_app::offline_mode::Connectivity::Down;
            while rx.changed().await.is_ok() {
                let status = *rx.borrow_and_update();
                let now_offline = status.is_offline();
                let now_conn_down =
                    status.connectivity == qbz_app::offline_mode::Connectivity::Down;
                let conn_changed = now_conn_down != was_conn_down;
                was_conn_down = now_conn_down;
                if now_offline == was_offline {
                    // Connectivity flipped WITHOUT a mode change (e.g. the
                    // link dying or returning during a logged-out session):
                    // the connectivity-keyed network-folder gate changes the
                    // browse SET, so refresh LocalLibrary in place.
                    if conn_changed {
                        let runtime2 = runtime.clone();
                        let nav_weak = weak.clone();
                        let handle2 = handle.clone();
                        let image_cache2 = image_cache.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            local_library::reset_browse_models(&w);
                            if w.global::<NavState>().get_view() == ContentView::LocalLibrary {
                                let tab = local_library::LibTab::from_tab_id(
                                    &w.global::<LocalLibraryState>().get_active_tab(),
                                )
                                .unwrap_or(local_library::LibTab::Albums);
                                navigate_local_library(
                                    runtime2, nav_weak, &handle2, image_cache2, tab,
                                );
                            }
                        });
                    }
                    continue;
                }
                was_offline = now_offline;
                if !now_offline {
                    // Back online: refresh the sidebar with the real Qobuz set
                    // (the offline cache may hold synthesized names).
                    load_sidebar_playlists(runtime.clone(), weak.clone(), &handle);
                    // Drop the LocalLibrary browse sets so the next visit
                    // re-fetches under the new state (the connectivity-keyed
                    // network-folder gate may change the SET), and reload in
                    // place when the user is standing there.
                    let runtime2 = runtime.clone();
                    let nav_weak = weak.clone();
                    let handle2 = handle.clone();
                    let image_cache2 = image_cache.clone();
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        local_library::reset_browse_models(&w);
                        if w.global::<NavState>().get_view() == ContentView::LocalLibrary {
                            let tab = local_library::LibTab::from_tab_id(
                                &w.global::<LocalLibraryState>().get_active_tab(),
                            )
                            .unwrap_or(local_library::LibTab::Albums);
                            navigate_local_library(
                                runtime2,
                                nav_weak,
                                &handle2,
                                image_cache2,
                                tab,
                            );
                        }
                    });
                    continue;
                }
                let runtime = runtime.clone();
                let nav_weak = weak.clone();
                let handle2 = handle.clone();
                let image_cache = image_cache.clone();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    // Sidebar: re-render from cache under the new offline state
                    // (the D11.b filter lives in sidebar::rebuild).
                    sidebar::rebuild(&w);
                    refresh_sidebar_covers(&w);
                    // Drop the browse sets so the next fetch (incl. the D12b
                    // navigation below) re-derives under offline. The SET is
                    // identical (network content is never hidden); the reset
                    // only refreshes per-row availability chrome.
                    local_library::reset_browse_models(&w);
                    match w.global::<NavState>().get_view() {
                        // D11.c: refresh the open grid/detail so unavailable
                        // items (and all-unavailable collections) drop.
                        ContentView::Mixtapes => {
                            myqbz::navigate(
                                nav_weak.clone(),
                                handle2.clone(),
                                image_cache.clone(),
                                qbz_models::mixtape::CollectionKind::Mixtape,
                            );
                        }
                        ContentView::Collections => {
                            myqbz::navigate(
                                nav_weak.clone(),
                                handle2.clone(),
                                image_cache.clone(),
                                qbz_models::mixtape::CollectionKind::Collection,
                            );
                        }
                        ContentView::MixtapeDetail => {
                            let id = w.global::<MyQbzDetailState>().get_id().to_string();
                            if !id.is_empty() {
                                myqbz_detail::navigate(
                                    runtime.clone(),
                                    nav_weak.clone(),
                                    handle2.clone(),
                                    image_cache.clone(),
                                    id,
                                );
                            }
                        }
                        ContentView::LocalLibrary => {
                            // Standing on a browse tab: the models were just
                            // reset — reload the active tab in place so the
                            // grid re-fetches under the offline gate instead
                            // of sitting empty until re-entry.
                            let tab = local_library::LibTab::from_tab_id(
                                &w.global::<LocalLibraryState>().get_active_tab(),
                            )
                            .unwrap_or(local_library::LibTab::Albums);
                            navigate_local_library(
                                runtime.clone(),
                                nav_weak.clone(),
                                &handle2,
                                image_cache.clone(),
                                tab,
                            );
                        }
                        _ => {
                            // D12b: blocked Qobuz view → LocalLibrary.
                            if is_offline_blocked_view(&w) {
                                nav::record(nav::NavEntry::LocalLibrary {
                                    tab: local_library::LibTab::Albums.tab_id().to_string(),
                                });
                                update_nav_flags(&w);
                                navigate_local_library(
                                    runtime.clone(),
                                    nav_weak.clone(),
                                    &handle2,
                                    image_cache.clone(),
                                    local_library::LibTab::Albums,
                                );
                            }
                        }
                    }
                });
            }
        });
    }

    // Startup: initialize the core, then try to restore a saved session.
    // A valid saved token jumps straight to the shell; otherwise the
    // login screen stays.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        let settings_ctx = settings_ctx.clone();
        tokio_rt.spawn(async move {
            if let Err(e) = runtime.init().await {
                log::error!("[qbz-slint] core init failed: {e}");
            }
            match auth::restore_saved_session(&runtime).await {
                Ok(Some(session)) => {
                    log::info!(
                        "[qbz-slint] session restored for user {}",
                        session.user_id
                    );
                    enter_shell(runtime, weak, image_cache, settings_ctx, session).await;
                }
                Ok(None) => {
                    log::info!("[qbz-slint] no saved session — showing login");
                    let _ = weak.upgrade_in_event_loop(|w| w.set_screen(AppScreen::Login));
                }
                Err(e) => {
                    log::error!("[qbz-slint] session restore failed: {e}");
                    // Surface the failure on the login screen (init-error box,
                    // spec §4.1); cleared again on any successful shell entry.
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        w.global::<OfflineState>().set_login_error(e.into());
                        w.set_screen(AppScreen::Login);
                    });
                }
            }
        });
    }

    // Sign in via the system browser → real OAuth → shell. The app has no
    // embedded webview: the one blue button opens the default browser, and
    // LoginState narrates the flow (waiting / authenticating / error) so the
    // login screen never sits inert while the OAuth is pending.
    // The in-flight task is kept so the screen's Cancel link can abort it
    // (dropping the task drops the one-shot listener and frees the port).
    let login_task: Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>> =
        Arc::new(std::sync::Mutex::new(None));
    let on_browser_login = {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let settings_ctx = settings_ctx.clone();
        let login_task = login_task.clone();
        move || {
            // Runs on the UI thread: flip to the waiting state immediately so
            // a second click cannot start a parallel flow, and clear any
            // previous failure.
            if let Some(w) = weak.upgrade() {
                let login_state = w.global::<LoginState>();
                if login_state.get_phase() != 0 {
                    return;
                }
                login_state.set_error("".into());
                // A new sign-in supersedes any stale boot-restore error —
                // without this both error boxes can show at once (review fix).
                w.global::<OfflineState>().set_login_error("".into());
                login_state.set_phase(1);
            }
            let runtime = runtime.clone();
            let weak = weak.clone();
            let image_cache = image_cache.clone();
            let settings_ctx = settings_ctx.clone();
            let task = handle.spawn(async move {
                let phase_weak = weak.clone();
                let result = auth::login_via_system_browser(&runtime, move |phase| {
                    let value = match phase {
                        auth::LoginPhase::WaitingForBrowser => 1,
                        auth::LoginPhase::Authenticating => 2,
                    };
                    let _ = phase_weak.upgrade_in_event_loop(move |w| {
                        w.global::<LoginState>().set_phase(value);
                    });
                })
                .await;
                match result {
                    Ok(session) => {
                        log::info!(
                            "[qbz-slint] authenticated as user {}",
                            session.user_id
                        );
                        enter_shell(runtime, weak, image_cache, settings_ctx, session).await;
                    }
                    Err(e) => {
                        log::error!("[qbz-slint] sign-in failed: {e}");
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            let login_state = w.global::<LoginState>();
                            login_state.set_phase(0);
                            login_state.set_error(e.into());
                        });
                    }
                }
            });
            *login_task.lock().unwrap() = Some(task);
        }
    };

    {
        let login = on_browser_login.clone();
        window.on_sign_in_via_browser(move || {
            dispatch(AppCommand::SignInViaBrowser);
            login();
        });
    }

    // Cancel link on the login screen (visible only while waiting for the
    // browser): abort the pending OAuth task and return to idle. Aborting
    // drops the local listener; the browser tab just fails to redirect.
    {
        let weak = window.as_weak();
        let login_task = login_task.clone();
        window.on_cancel_login(move || {
            if let Some(task) = login_task.lock().unwrap().take() {
                task.abort();
            }
            if let Some(w) = weak.upgrade() {
                let login_state = w.global::<LoginState>();
                login_state.set_phase(0);
                login_state.set_error("".into());
            }
            log::info!("[qbz-slint] browser sign-in cancelled by user");
        });
    }

    // Offline: enter a full offline session at the last user (local library,
    // offline cache, settings — no Qobuz auth), then show the shell.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let settings_ctx = settings_ctx.clone();
        window.on_start_offline(move || {
            dispatch(AppCommand::StartOffline);
            let runtime = runtime.clone();
            let weak = weak.clone();
            let image_cache = image_cache.clone();
            let settings_ctx = settings_ctx.clone();
            handle.spawn(async move {
                if let Err(e) = enter_shell_offline(runtime, weak, image_cache, settings_ctx).await
                {
                    log::error!("[qbz-slint] offline start failed: {e}");
                }
            });
        });
    }

    // D2 recovery: one click on the shell banner re-logs-in with the saved
    // token and runs the full online entry over the live offline session.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        let settings_ctx = settings_ctx.clone();
        let login_task = login_task.clone();
        window.on_recovery_login(move || {
            // Logged BEFORE the spawn: records the click arriving from the
            // UI chain even if the async attempt below stalls or fails.
            log::info!("[qbz-slint] recovery sign-in requested");
            let runtime = runtime.clone();
            let weak = weak.clone();
            let image_cache = image_cache.clone();
            let settings_ctx = settings_ctx.clone();
            let task = handle.spawn(async move {
                // No pre-lift anywhere: the auth endpoints are EXEMPT from
                // the offline gate (qbz-qobuz client), so the token login and
                // the OAuth exchange pass the closed gate — and
                // login_via_system_browser no longer clears offline_session
                // up front either. The flag ends up false only on SUCCESS
                // paths (restore_saved_session / login_via_system_browser
                // clear it after the login completes), so the shell never
                // sits unlocked-and-empty while an attempt is pending, and a
                // failed attempt leaves the live offline session intact.
                match auth::restore_saved_session(&runtime).await {
                    Ok(Some(session)) => {
                        log::info!(
                            "[qbz-slint] recovery login succeeded for user {}",
                            session.user_id
                        );
                        enter_shell(runtime, weak, image_cache, settings_ctx, session).await;
                    }
                    Ok(None) => {
                        // No saved token, or the token was explicitly
                        // rejected (and cleared). The user asked to sign in —
                        // fall back to the full system-browser OAuth. Show
                        // the LOGIN screen FIRST: its UX narrates the
                        // browser flow (the user shouldn't have to notice
                        // the opened browser on their own), and it replaces
                        // the offline shell instead of leaving it on screen
                        // while the attempt runs.
                        log::warn!(
                            "[qbz-slint] recovery login: saved session unusable — falling back to browser OAuth"
                        );
                        let _ = weak.upgrade_in_event_loop(|w| {
                            // Seed the waiting narration before the browser
                            // opens so the screen never shows an idle button
                            // while the flow is already running.
                            let login_state = w.global::<LoginState>();
                            login_state.set_error("".into());
                            login_state.set_phase(1);
                            w.set_screen(AppScreen::Login);
                        });
                        let phase_weak = weak.clone();
                        let login_result =
                            auth::login_via_system_browser(&runtime, move |phase| {
                                let value = match phase {
                                    auth::LoginPhase::WaitingForBrowser => 1,
                                    auth::LoginPhase::Authenticating => 2,
                                };
                                let _ = phase_weak.upgrade_in_event_loop(move |w| {
                                    w.global::<LoginState>().set_phase(value);
                                });
                            })
                            .await;
                        match login_result {
                            Ok(session) => {
                                log::info!(
                                    "[qbz-slint] recovery browser sign-in succeeded for user {}",
                                    session.user_id
                                );
                                enter_shell(runtime, weak, image_cache, settings_ctx, session)
                                    .await;
                            }
                            Err(e) => {
                                log::error!("[qbz-slint] recovery browser sign-in failed: {e}");
                                // The offline session was never lifted, so
                                // there is nothing to restore. Stay on the
                                // Login screen: the error box explains the
                                // failure, and the "Start offline" link
                                // (has-previous-session) leads back into
                                // the offline shell.
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    toast::error(&w, format!("Sign-in failed: {e}"));
                                    let login_state = w.global::<LoginState>();
                                    login_state.set_phase(0);
                                    login_state.set_error(e.into());
                                });
                            }
                        }
                    }
                    Err(e) => {
                        // Init-class failure (gated/unreachable cold bundle
                        // fetch): any transient flag lift was already undone
                        // inside auth, so the offline shell state is intact —
                        // just surface the error.
                        log::error!("[qbz-slint] recovery login failed: {e}");
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            toast::error(&w, format!("Sign-in failed: {e}"));
                            w.global::<OfflineState>().set_login_error(e.into());
                        });
                    }
                }
            });
            // Same slot the login screen's Cancel link aborts — the browser
            // leg of this recovery flow is cancellable like a normal sign-in.
            *login_task.lock().unwrap() = Some(task);
        });
    }


    // Open an album: record history, then load and show it.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_open_album(move |album_id| {
            let album_id = album_id.to_string();
            // A local item carries a metadata group key, not a Qobuz id —
            // route it to the LocalAlbum view (Home "Recently played", the
            // now-playing bar's "Go to album", etc.) instead of the empty
            // Qobuz album view.
            if is_local_album_key(&album_id) {
                nav::record(nav::NavEntry::LocalAlbum(album_id.clone()));
                navigate_local_album(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    album_id,
                );
            } else {
                // Feed Capa B if this Qobuz album was opened from the search
                // results page (gated inside the helper). Local-album keys take
                // the branch above and never reach here.
                if let Some(w) = weak.upgrade() {
                    record_search_interaction(
                        &w,
                        "album",
                        &album_id,
                        crate::search_service::InteractionAction::Open,
                    );
                }
                nav::record(nav::NavEntry::Album(album_id.clone()));
                navigate_album(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    album_id,
                );
            }
            if let Some(w) = weak.upgrade() {
                update_nav_flags(&w);
            }
        });
    }

    // Open an artist: record history, then load and show the page.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_open_artist(move |artist_ref| {
            let artist_ref = artist_ref.to_string();
            // Qobuz artists are numeric ids → the Qobuz artist page. Local
            // artists have no id, so their surfaces (LocalAlbum link, now-playing
            // "Go to artist") pass the NAME instead → the LocalLibrary Artists
            // tab, focused on that artist.
            if artist_ref.parse::<u64>().is_ok() {
                // Feed Capa B if this Qobuz artist was opened from the search
                // results page (gated inside the helper). Local artists
                // pass a NAME (non-numeric) and take the branch below — never
                // recorded.
                if let Some(w) = weak.upgrade() {
                    record_search_interaction(
                        &w,
                        "artist",
                        &artist_ref,
                        crate::search_service::InteractionAction::Open,
                    );
                }
                nav::record(nav::NavEntry::Artist(artist_ref.clone()));
                navigate_artist(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    artist_ref,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            } else if !artist_ref.trim().is_empty() {
                open_local_artist(&runtime, &weak, &handle, &image_cache, artist_ref);
            }
        });
    }

    // Submit search (Enter): record history and show the results page.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<SearchActions>().on_submit(move |query| {
            let q = query.trim().to_string();
            if q.len() < 2 {
                return;
            }
            SEARCH_DEBOUNCE.with(|t| t.stop());
            nav::push_or_replace_search(q.clone());
            navigate_search(runtime.clone(), weak.clone(), &handle, image_cache.clone(), q);
            if let Some(w) = weak.upgrade() {
                // Enter -> results page: dismiss the live dropdown and always
                // land on Search > All (never a lingering per-type tab).
                let st = w.global::<SearchState>();
                st.set_cortinilla_open(false);
                st.set_tab(0);
                update_nav_flags(&w);
            }
        });
    }

    // Live search: debounce 300 ms, minimum 2 characters. Does not record
    // history (per-keystroke entries would pollute the back stack).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<SearchActions>().on_live(move |query| {
            let q = query.trim().to_string();
            // chars().count(): the >= 2 gate is on grapheme-ish length, not
            // bytes, so a 2-char multibyte query (e.g. CJK) is not rejected.
            if q.chars().count() < 2 {
                SEARCH_DEBOUNCE.with(|t| t.stop());
                // Below the threshold — close the cortinilla so a backspaced
                // query does not leave a stale dropdown open.
                if let Some(w) = weak.upgrade() {
                    w.global::<SearchState>().set_cortinilla_open(false);
                }
                return;
            }

            // --- Cortinilla (live dropdown), only when the module is ON (D5) ---
            // The results-page debounce below is untouched; the cortinilla is a
            // separate, additive surface gated on the kill switch.
            if crate::search_service::is_enabled() {
                let expand_local = if let Some(w) = weak.upgrade() {
                    let st = w.global::<SearchState>();
                    // Always reset the keyboard selection + scroll on (re)open or
                    // refine — never leave a stale "active row" from a prior
                    // search. Arrow nav fires no keystroke, so it is unaffected.
                    st.set_selected_index(-1);
                    st.set_cortinilla_scroll_y(0.0);
                    st.set_cortinilla_open(true);
                    st.set_cortinilla_query(q.clone().into());
                    st.set_cortinilla_loading(true);
                    // Offline OR an unauthenticated (offline) session → the Qobuz
                    // half is empty, so the dropdown is local-only; widen the
                    // on-device section caps.
                    let off = w.global::<OfflineState>();
                    off.get_offline() || off.get_offline_session()
                } else {
                    false
                };
                let cort_version = search::next_cortinilla_version();

                // No cached instant-paint. The cached -> fresh swap (plus the
                // local-fold mid-apply) made the results visibly "jump". Instead
                // the placeholder skeleton (cortinilla-loading) shows while typing
                // and a SINGLE apply paints the real results ~220 ms after the
                // last keystroke — debounced so rapid typing fires one load, not
                // one per keystroke. The version guard drops any stale in-flight
                // load; `load_cortinilla` already folds the on-device section in,
                // so this is one combined paint with no intermediate states.
                {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    let image_cache = image_cache.clone();
                    let q = q.clone();
                    CORTINILLA_DEBOUNCE.with(|t| {
                        t.start(
                            slint::TimerMode::SingleShot,
                            std::time::Duration::from_millis(220),
                            move || {
                                let runtime = runtime.clone();
                                let weak = weak.clone();
                                let image_cache = image_cache.clone();
                                let q = q.clone();
                                handle.spawn(async move {
                                    match search::load_cortinilla(&runtime, &q, expand_local).await {
                                        Ok((data, local_rows)) => {
                                            let jobs = search::cortinilla_artwork_jobs(&data);
                                            let _ = weak.clone().upgrade_in_event_loop(move |w| {
                                                if search::is_current_cortinilla_version(cort_version) {
                                                    LAST_CORTINILLA.with(|c| {
                                                        *c.borrow_mut() = Some(data.clone())
                                                    });
                                                    LAST_CORTINILLA_LOCAL
                                                        .with(|c| *c.borrow_mut() = local_rows);
                                                    search::apply_cortinilla(&w, data);
                                                }
                                            });
                                            // Mixed payload (Qobuz http / local fs) —
                                            // route each cover by scheme.
                                            artwork::spawn_search_loads(
                                                jobs,
                                                weak.clone(),
                                                image_cache,
                                            );
                                        }
                                        Err(e) => {
                                            log::error!("[qbz-slint] cortinilla load failed: {e}");
                                            let _ = weak.upgrade_in_event_loop(move |w| {
                                                if search::is_current_cortinilla_version(cort_version) {
                                                    w.global::<SearchState>()
                                                        .set_cortinilla_loading(false);
                                                }
                                            });
                                        }
                                    }
                                });
                            },
                        );
                    });
                }
            }

            // --- Results page LIVE search — ONLY when the module is OFF --------
            // When Intelligent Search is ON, the cortinilla above is the live
            // preview; typing must NOT auto-navigate to the results page. The
            // 300 ms debounce-navigate would otherwise hijack navigation — a
            // pending fire lands on the results page ~300 ms after the last
            // keystroke and overrides wherever the user just went (e.g. a
            // cortinilla row-click), so "I can't navigate anywhere, it takes me
            // to the result". Enter (on_submit) still navigates there. When the
            // module is OFF, keep the Phase-1 live-results behavior unchanged.
            if crate::search_service::is_enabled() {
                SEARCH_DEBOUNCE.with(|t| t.stop());
                return;
            }
            // --- Results page (module OFF): debounce 300 ms, then full search ---
            let runtime = runtime.clone();
            let weak = weak.clone();
            let handle = handle.clone();
            let image_cache = image_cache.clone();
            SEARCH_DEBOUNCE.with(|t| {
                t.start(
                    slint::TimerMode::SingleShot,
                    std::time::Duration::from_millis(300),
                    move || {
                        // Record (or replace) the Search history entry so
                        // back/forward returns to this search instead of
                        // skipping past it.
                        nav::push_or_replace_search(q.clone());
                        navigate_search(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            q.clone(),
                        );
                        if let Some(w) = weak.upgrade() {
                            update_nav_flags(&w);
                        }
                    },
                );
            });
        });
    }

    // Switch search results tab. search_all already loaded every
    // category, so this only changes which one the view renders.
    {
        let weak = window.as_weak();
        window.global::<SearchActions>().on_tab_changed(move |tab| {
            if let Some(w) = weak.upgrade() {
                w.global::<SearchState>().set_tab(tab);
            }
        });
    }

    // Load more results for the active per-type tab. The offset is the
    // count already loaded into that category's list.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<SearchActions>().on_load_more(move |tab| {
            let Some(w) = weak.upgrade() else {
                return;
            };
            let Some(category) = search::category_for_tab(tab) else {
                return;
            };
            let st = w.global::<SearchState>();
            let query = st.get_query().to_string();
            let filter = search::search_type_for_filter(st.get_filter_index());
            let offset = match category {
                search::SearchCategory::Albums => st.get_albums().row_count(),
                search::SearchCategory::Tracks => st.get_tracks().row_count(),
                search::SearchCategory::Artists => st.get_artists().row_count(),
                search::SearchCategory::Playlists => st.get_playlists().row_count(),
            } as u32;
            let runtime = runtime.clone();
            let weak = weak.clone();
            let image_cache = image_cache.clone();
            handle.spawn(async move {
                match search::load_more(&runtime, &query, category, filter, offset).await {
                    Ok(more) => {
                        let jobs = search::artwork_jobs_for_more(&more, offset as usize);
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            search::append_results(&w, more);
                        });
                        artwork::spawn_loads(jobs, weak.clone(), image_cache);
                    }
                    Err(e) => log::error!("[qbz-slint] search load-more failed: {e}"),
                }
            });
        });
    }

    // Change the searchType filter: re-query the three filterable
    // categories (albums / tracks / artists) and replace their lists, so
    // the filter takes effect on every tab including All.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<SearchActions>().on_filter_changed(move |index| {
            let Some(w) = weak.upgrade() else {
                return;
            };
            let st = w.global::<SearchState>();
            st.set_filter_index(index);
            let query = st.get_query().to_string();
            if query.trim().is_empty() {
                return;
            }
            let search_type = search::search_type_for_filter(index);
            let runtime = runtime.clone();
            let weak = weak.clone();
            let image_cache = image_cache.clone();
            handle.spawn(async move {
                for category in [
                    search::SearchCategory::Albums,
                    search::SearchCategory::Tracks,
                    search::SearchCategory::Artists,
                ] {
                    match search::load_more(&runtime, &query, category, search_type.clone(), 0)
                        .await
                    {
                        Ok(more) => {
                            let jobs = search::artwork_jobs_for_more(&more, 0);
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                search::replace_category(&w, more);
                            });
                            artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                        }
                        Err(e) => log::error!("[qbz-slint] search filter failed: {e}"),
                    }
                }
            });
        });
    }

    // "Hi-Res only" toggle: pure client-side re-filter of the already-loaded
    // albums/tracks — no re-fetch. The `bool` arg mirrors LabelActions'
    // on_set_hires (state is already flipped by the ToggleButton itself
    // before this fires; see the toolbar in SearchResultsView.slint).
    // Qobuz's search endpoints take no quality parameter
    // (search::recompute_hi_res_filtered has the full rationale), so unlike
    // on_filter_changed above this never spawns a network task.
    {
        let weak = window.as_weak();
        window.global::<SearchActions>().on_hires_only_changed(move |_| {
            if let Some(w) = weak.upgrade() {
                search::recompute_hi_res_filtered(&w);
            }
        });
    }

    // Cortinilla: dismiss (click-outside / Escape).
    {
        let weak = window.as_weak();
        window.global::<SearchActions>().on_cortinilla_dismiss(move || {
            if let Some(w) = weak.upgrade() {
                let st = w.global::<SearchState>();
                st.set_cortinilla_open(false);
                // Clear the keyboard/hover highlight too — a dismissed dropdown
                // has no meaningful selection, and the `changed view` close-hook
                // (AppShell) relies on this to reset the highlight on navigation.
                st.set_selected_index(-1);
            }
        });
    }

    // Cortinilla: arrow-key move the keyboard highlight (delta -1 up / +1 down).
    // The valid navigable flat indices are NOT guaranteed to be a contiguous
    // 0..=max range (when there is no top result, index 0 is skipped and the
    // section rows start at 1), so the order is rebuilt from the live snapshot:
    // the top-result's flat index first (when present), then every section row's
    // flat index in declaration order. `selected-index == -1` means "nothing
    // highlighted" (Enter falls through to search-all); Down from -1 lands on the
    // first row, Up from the first row returns to -1. Both ends clamp (no wrap).
    {
        let weak = window.as_weak();
        window
            .global::<SearchActions>()
            .on_cortinilla_move_selection(move |delta| {
                let Some(w) = weak.upgrade() else { return };
                // Build the ordered list of navigable flat indices.
                let order: Vec<i32> = LAST_CORTINILLA.with(|c| {
                    let snap = c.borrow();
                    let Some(data) = snap.as_ref() else {
                        return Vec::new();
                    };
                    let mut v: Vec<i32> = Vec::new();
                    if let Some(top) = &data.top {
                        v.push(top.flat_index as i32);
                    }
                    for section in &data.sections {
                        for row in &section.rows {
                            v.push(row.flat_index as i32);
                        }
                    }
                    v
                });
                if order.is_empty() {
                    return;
                }
                let st = w.global::<SearchState>();
                let current = st.get_selected_index();
                // Current position within the navigable order (-1 if nothing /
                // stale value not present anymore).
                let pos = order.iter().position(|&fi| fi == current);
                let new_index: i32 = if delta > 0 {
                    // Down: from "nothing" -> first; otherwise advance, clamping
                    // at the last row.
                    match pos {
                        None => order[0],
                        Some(p) if p + 1 < order.len() => order[p + 1],
                        Some(_) => order[order.len() - 1],
                    }
                } else {
                    // Up: from "nothing" stay nothing; from the first row -> -1;
                    // otherwise step back.
                    match pos {
                        None => -1,
                        Some(0) => -1,
                        Some(p) => order[p - 1],
                    }
                };
                st.set_selected_index(new_index);
                // Content-top y of the selected row so the overlay can scroll it
                // into view. Mirrors Cortinilla.slint's layout EXACTLY: top-result
                // block = padTop(4) + label(22) + row(56); each section block =
                // padTop(4) + header(24) + rows × 56. 0 when nothing is selected.
                let scroll_y: f32 = if new_index < 0 {
                    0.0
                } else {
                    LAST_CORTINILLA.with(|c| {
                        let snap = c.borrow();
                        let Some(data) = snap.as_ref() else {
                            return 0.0;
                        };
                        let mut y: f32 = 0.0;
                        if let Some(top) = &data.top {
                            if top.flat_index as i32 == new_index {
                                return y + 26.0; // padTop 4 + label 22
                            }
                            y += 82.0; // 4 + 22 + 56
                        }
                        for section in &data.sections {
                            y += 28.0; // padTop 4 + header 24
                            for row in &section.rows {
                                if row.flat_index as i32 == new_index {
                                    return y;
                                }
                                y += 56.0;
                            }
                        }
                        0.0
                    })
                };
                st.set_cortinilla_scroll_y(scroll_y);
            });
    }

    // Cortinilla: Enter with nothing highlighted — run a full search-all on the
    // current live query (same path as submit) and dismiss the dropdown.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SearchActions>()
            .on_cortinilla_search_all(move || {
                let Some(w) = weak.upgrade() else { return };
                let q = w
                    .global::<SearchState>()
                    .get_cortinilla_query()
                    .trim()
                    .to_string();
                if q.chars().count() < 2 {
                    return;
                }
                let st = w.global::<SearchState>();
                st.set_cortinilla_open(false);
                // Activating the cortinilla's Enter affordance clears the input
                // too (consistent with row-click / View-more), so it can't
                // re-invoke the dropdown over the results page.
                st.set_header_search_text("".into());
                // Enter always lands on Search > All, never a per-type tab.
                st.set_tab(0);
                SEARCH_DEBOUNCE.with(|t| t.stop());
                nav::push_or_replace_search(q.clone());
                navigate_search(runtime.clone(), weak.clone(), &handle, image_cache.clone(), q);
                update_nav_flags(&w);
            });
    }

    // Cortinilla: "View more" on a section. Qobuz categories open the full
    // results page on the matching tab (albums=1, tracks=2, artists=3,
    // playlists=4); the "local" section opens the LocalLibrary Tracks tab
    // pre-filtered to the live query (local results never enter the Qobuz
    // results page — D1/D2).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SearchActions>()
            .on_cortinilla_view_more(move |kind| {
                let Some(w) = weak.upgrade() else { return };
                let kind = kind.to_string();
                let q = w
                    .global::<SearchState>()
                    .get_cortinilla_query()
                    .trim()
                    .to_string();
                if q.chars().count() < 2 {
                    return;
                }
                {
                    let st = w.global::<SearchState>();
                    st.set_cortinilla_open(false);
                    // Clear the input so it can't re-invoke the dropdown later.
                    st.set_header_search_text("".into());
                }
                SEARCH_DEBOUNCE.with(|t| t.stop());

                // On-device "View more": leave the Qobuz results page entirely
                // and open the matching LocalLibrary tab pre-filtered to the live
                // query (D1/D2: local results never live in the Qobuz results
                // page). Albums / Artists / Tracks each route to their own tab,
                // setting that tab's search filter then forcing a re-derive so the
                // filtered set renders on both first-visit and re-entry.
                if kind == "local-album" {
                    w.global::<LocalLibraryState>().set_albums_search(q.clone().into());
                    nav::record(nav::NavEntry::LocalLibrary {
                        tab: local_library::LibTab::Albums.tab_id().to_string(),
                    });
                    navigate_local_library(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        local_library::LibTab::Albums,
                    );
                    // Force a reload so the freshly-set search filter applies even
                    // when the Albums tab was already loaded (re-entry).
                    local_library::reload_albums(weak.clone(), handle.clone(), image_cache.clone());
                    update_nav_flags(&w);
                    return;
                }
                if kind == "local-artist" {
                    w.global::<LocalLibraryState>().set_artists_search(q.clone().into());
                    nav::record(nav::NavEntry::LocalLibrary {
                        tab: local_library::LibTab::Artists.tab_id().to_string(),
                    });
                    navigate_local_library(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        local_library::LibTab::Artists,
                    );
                    // Re-derive in place so the filter applies on re-entry (the
                    // async first-load re-derives with the same filter on its own).
                    local_library::derive_artists(&w);
                    update_nav_flags(&w);
                    return;
                }
                if kind == "local" {
                    w.global::<LocalLibraryState>().set_tracks_search(q.clone().into());
                    nav::record(nav::NavEntry::LocalLibrary {
                        tab: local_library::LibTab::Tracks.tab_id().to_string(),
                    });
                    navigate_local_library(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        local_library::LibTab::Tracks,
                    );
                    // `navigate_local_library` only lazy-loads on an EMPTY tracks
                    // model (re-entry keeps the prior set), so force a reload to
                    // apply the freshly-set search filter regardless.
                    local_library::reload_tracks(weak.clone(), handle.clone());
                    update_nav_flags(&w);
                    return;
                }

                // Qobuz category → open the full results page on the matching tab.
                let tab = match kind.as_str() {
                    "album" => 1,
                    "track" => 2,
                    "artist" => 3,
                    "playlist" => 4,
                    _ => 0,
                };
                nav::push_or_replace_search(q.clone());
                navigate_search(runtime.clone(), weak.clone(), &handle, image_cache.clone(), q);
                // search_all loads every category; the tab switch only changes
                // which list renders. Apply it after navigate so it sticks.
                w.global::<SearchState>().set_tab(tab);
                update_nav_flags(&w);
            });
    }

    // Cortinilla: a row was activated (click or Enter on a highlight). Resolve
    // the flat index against the controller snapshot, then dispatch to the SAME
    // nav/play seams the results page uses.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SearchActions>()
            .on_cortinilla_row_clicked(move |flat_index| {
                let Some(w) = weak.upgrade() else { return };
                // Resolve flat_index -> the concrete row from the snapshot.
                let row = LAST_CORTINILLA.with(|c| {
                    let snap = c.borrow();
                    let data = snap.as_ref()?;
                    if let Some(top) = &data.top {
                        if top.flat_index as i32 == flat_index {
                            return Some(top.clone());
                        }
                    }
                    data.sections
                        .iter()
                        .flat_map(|s| s.rows.iter())
                        .find(|r| r.flat_index as i32 == flat_index)
                        .cloned()
                });
                let Some(row) = row else { return };

                // Capture the live cortinilla query BEFORE dismissing so the
                // ranking feedback (Capa B) is keyed off the query that produced
                // this row.
                let cort_query = w
                    .global::<SearchState>()
                    .get_cortinilla_query()
                    .to_string();

                // Dismiss the dropdown before acting AND clear the header input —
                // once the user activates a row, leftover text would otherwise
                // re-invoke the cortinilla when focus bounces back to the field.
                {
                    let st = w.global::<SearchState>();
                    st.set_cortinilla_open(false);
                    st.set_header_search_text("".into());
                }

                // Feed Capa B: a clicked QOBUZ row is an interaction with the
                // search-surfaced entity. action = Play for tracks (they play on
                // click), Open for album/artist/playlist (they navigate). LOCAL
                // rows are intentionally NOT recorded — local entities use a
                // different id space (D4) and are skipped in v1. record() no-ops
                // when the module is disabled, so the unconditional call is safe.
                if row.source != "local" {
                    let action = if row.kind == "track" {
                        crate::search_service::InteractionAction::Play
                    } else {
                        crate::search_service::InteractionAction::Open
                    };
                    crate::search_service::record(&cort_query, &row.kind, &row.id, action);
                }

                if row.source == "local" {
                    // On-device rows route by kind (the "links go to LocalLibrary"
                    // requirement): a local ALBUM opens the LocalAlbum view by its
                    // group key; a local ARTIST opens the LocalLibrary Artists tab
                    // by NAME (local artists have no id); a local TRACK plays
                    // through the LOCAL seam.
                    match row.kind.as_str() {
                        "album" => {
                            // `row.id` is the album_group_key (a local album key).
                            let key = row.id.clone();
                            nav::record(nav::NavEntry::LocalAlbum(key.clone()));
                            navigate_local_album(
                                runtime.clone(),
                                weak.clone(),
                                &handle,
                                image_cache.clone(),
                                key,
                            );
                            update_nav_flags(&w);
                        }
                        "artist" => {
                            // Local artists are keyed by NAME (`row.title`).
                            open_local_artist(
                                &runtime,
                                &weak,
                                &handle,
                                &image_cache,
                                row.title.clone(),
                            );
                        }
                        _ => {
                            // Track: play this on-device row + its siblings (so the
                            // queue continues down the list), starting at the
                            // clicked one. `row.id` is the library row id.
                            let tracks = LAST_CORTINILLA_LOCAL.with(|c| c.borrow().clone());
                            let start = tracks
                                .iter()
                                .position(|t| t.id.to_string() == row.id)
                                .unwrap_or(0);
                            if !tracks.is_empty() {
                                playback::play_local_tracks(
                                    runtime.clone(),
                                    weak.clone(),
                                    handle.clone(),
                                    tracks,
                                    start,
                                    false,
                                );
                            }
                        }
                    }
                    return;
                }

                match row.kind.as_str() {
                    "album" => {
                        let id = row.id.clone();
                        nav::record(nav::NavEntry::Album(id.clone()));
                        navigate_album(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id,
                        );
                        update_nav_flags(&w);
                    }
                    "artist" => {
                        let id = row.id.clone();
                        nav::record(nav::NavEntry::Artist(id.clone()));
                        navigate_artist(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id,
                        );
                        update_nav_flags(&w);
                    }
                    "playlist" => {
                        let id = row.id.clone();
                        nav::record(nav::NavEntry::Playlist(id.clone()));
                        navigate_playlist(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id,
                        );
                        update_nav_flags(&w);
                    }
                    "track" => {
                        // A clicked Qobuz track plays immediately (single-track
                        // queue), matching the results-row "play".
                        if let Ok(track_id) = row.id.parse::<u64>() {
                            playback::play_track_now(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                track_id,
                            );
                        }
                    }
                    _ => {}
                }
            });
    }

    // History navigation — back / forward / settings, all recorded by the
    // nav module so the [<] [>] pair and the mouse buttons stay in sync.
    {
        let weak = window.as_weak();
        window.global::<NavState>().on_request_settings(move || {
            nav::record(nav::NavEntry::Settings);
            if let Some(w) = weak.upgrade() {
                seed_blacklist_status(&w);
                w.global::<NavState>().set_view(ContentView::Settings);
                update_nav_flags(&w);
            }
        });
    }

    // Keyboard shortcuts (hotkeys): seed the cheatsheet/editor model + wire the
    // customize-editor capture callbacks. The global key dispatch itself lives
    // in `install_browser_mouse_nav`'s winit handler.
    keybindings::wire(&window);

    // "Open Qobuz Link" (Ctrl+L) — the cross-platform link resolver modal.
    {
        let weak = window.as_weak();
        window
            .global::<LinkResolverActions>()
            .on_url_changed(move |url| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LinkResolverState>()
                        .set_platform(link_resolver::detect_platform(&url).into());
                }
            });
    }
    {
        let weak = window.as_weak();
        window.global::<LinkResolverActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                w.global::<LinkResolverState>().set_open(false);
            }
        });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LinkResolverActions>()
            .on_open_importer(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<LinkResolverState>().set_open(false);
                    w.global::<PlaylistImportState>().set_open(true);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<LinkResolverActions>().on_submit(move |url| {
            let url = url.trim().to_string();
            if url.is_empty() {
                return;
            }
            if let Some(w) = weak.upgrade() {
                let s = w.global::<LinkResolverState>();
                s.set_resolving(true);
                s.set_error("".into());
                s.set_playlist_detected(false);
            }
            let runtime = runtime.clone();
            let weak = weak.clone();
            let handle = handle.clone();
            let image_cache = image_cache.clone();
            handle.clone().spawn(async move {
                let result = link_resolver::resolve(runtime.clone(), url).await;
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let s = w.global::<LinkResolverState>();
                    s.set_resolving(false);
                    match result {
                        Ok(qbz_music_link::MusicLinkResult::Resolved { link, .. }) => {
                            s.set_open(false);
                            apply_resolved_link(
                                link,
                                &runtime,
                                &w.as_weak(),
                                &handle,
                                &image_cache,
                            );
                        }
                        Ok(qbz_music_link::MusicLinkResult::PlaylistDetected { provider }) => {
                            s.set_playlist_detected(true);
                            s.set_playlist_provider(provider.into());
                        }
                        Ok(qbz_music_link::MusicLinkResult::NotOnQobuz { .. }) => {
                            s.set_error(
                                qbz_i18n::t("This content is not available on Qobuz").into(),
                            );
                        }
                        Err(e) => {
                            log::warn!("[qbz-slint] open-link resolve failed: {e}");
                            s.set_error(qbz_i18n::t("Could not resolve that link").into());
                        }
                    }
                });
            });
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<NavState>().on_request_back(move || {
            if let Some((entry, scroll)) = nav::go_back() {
                arm_scroll_restore(&weak, &entry, scroll);
                apply_entry(entry, &runtime, &weak, &handle, &image_cache);
            }
            if let Some(w) = weak.upgrade() {
                update_nav_flags(&w);
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<NavState>().on_request_forward(move || {
            if let Some((entry, scroll)) = nav::go_forward() {
                arm_scroll_restore(&weak, &entry, scroll);
                apply_entry(entry, &runtime, &weak, &handle, &image_cache);
            }
            if let Some(w) = weak.upgrade() {
                update_nav_flags(&w);
            }
        });
    }
    {
        // The mounted scroll container reports its live viewport-y here so the
        // nav module can stamp the outgoing entry on the next navigation.
        window
            .global::<NavState>()
            .on_report_scroll(|y| nav::set_live_scroll(y));
    }

    // Log out: clear the session and return to the login screen.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.on_logout(move || {
            // Back at the login screen a pending deep link must wait for the
            // next enter_shell, not fire into the torn-down session.
            deep_link::clear_shell_ctx();
            let runtime = runtime.clone();
            let weak = weak.clone();
            handle.spawn(async move {
                if let Err(e) = auth::logout(&runtime).await {
                    log::error!("[qbz-slint] logout failed: {e}");
                }
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<NavState>().set_view(ContentView::Home);
                    w.global::<SessionState>().set_user_name("".into());
                    w.set_screen(AppScreen::Login);
                });
            });
        });
    }

    // Settings — a toggle changed: persist it and apply audio ones to the
    // live player.
    {
        let runtime = app_runtime.clone();
        let settings_ctx = settings_ctx.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.on_settings_bool(move |key, value| {
            let runtime = runtime.clone();
            let settings_ctx = settings_ctx.clone();
            let weak = weak.clone();
            let key = key.to_string();
            handle.spawn(async move {
                settings::handle_bool(settings_ctx, runtime, weak, key, value).await;
            });
        });
    }

    // Settings — a dropdown changed: persist it, apply audio ones, and
    // re-enumerate devices on a backend switch.
    {
        let runtime = app_runtime.clone();
        let settings_ctx = settings_ctx.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.on_settings_select(move |key, index| {
            let runtime = runtime.clone();
            let settings_ctx = settings_ctx.clone();
            let weak = weak.clone();
            let key = key.to_string();
            let index = index.max(0) as usize;
            handle.spawn(async move {
                settings::handle_select(settings_ctx, runtime, weak, key, index).await;
            });
        });
    }

    // Settings — a slider changed (Initial Buffer Size): persist it and
    // reload the player settings.
    {
        let runtime = app_runtime.clone();
        let settings_ctx = settings_ctx.clone();
        let handle = tokio_rt.handle().clone();
        window.on_settings_slider(move |key, value| {
            let runtime = runtime.clone();
            let settings_ctx = settings_ctx.clone();
            let key = key.to_string();
            handle.spawn(async move {
                settings::handle_slider(&settings_ctx, &runtime, &key, value);
            });
        });
    }

    // Settings — a text input committed.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.on_settings_string(move |key, value| {
            let weak = weak.clone();
            let key = key.to_string();
            let value = value.to_string();
            handle.spawn(async move {
                settings::handle_string(weak, key, value).await;
            });
        });
    }

    // Settings — Reset: restore Audio + Playback defaults, rebuild the
    // snapshot, and re-apply the audio settings to the player.
    {
        let runtime = app_runtime.clone();
        let settings_ctx = settings_ctx.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.on_settings_reset(move || {
            let runtime = runtime.clone();
            let settings_ctx = settings_ctx.clone();
            let weak = weak.clone();
            handle.spawn(async move {
                settings::handle_reset(settings_ctx, runtime, weak).await;
            });
        });
    }

    // Settings — the output-device refresh/release button: free a device QBZ
    // holds exclusively (ALSA Direct) and re-enumerate, so a freed or
    // hot-plugged DAC reappears without an app restart.
    {
        let runtime = app_runtime.clone();
        let settings_ctx = settings_ctx.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.on_settings_release_device(move || {
            let runtime = runtime.clone();
            let settings_ctx = settings_ctx.clone();
            let weak = weak.clone();
            handle.spawn(async move {
                settings::handle_release_device(settings_ctx, runtime, weak).await;
            });
        });
    }

    // Settings > Developer — "Export settings…" modal confirm: build the
    // settings bundle via the shared engine, open a native save dialog, write
    // it 0600, and toast the import command (04 §4.2). No new export logic.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<SettingsExportActions>().on_confirm(move || {
            settings::export_settings(weak.clone(), handle.clone());
        });
    }

    // Settings > Offline MODE — re-seed the toggle states on panel mount
    // (the panel's init fires load), and the status row's "Check now"
    // connectivity re-probe. The toggles themselves persist through the
    // generic settings-bool path above.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<OfflineModeActions>().on_load(move || {
            offline_mode::seed_settings(weak.clone(), handle.clone());
        });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<OfflineModeActions>().on_check_now(move || {
            offline_mode::check_now(weak.clone(), handle.clone());
        });
    }
    // The header badge flyout's quick offline toggle — same persistence +
    // #279 snapshot path as the Settings "Enable Offline Mode" toggle.
    {
        let runtime = app_runtime.clone();
        let settings_ctx = settings_ctx.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<OfflineModeActions>()
            .on_set_offline(move |value| {
                let runtime = runtime.clone();
                let settings_ctx = settings_ctx.clone();
                let weak = weak.clone();
                handle.spawn(async move {
                    settings::handle_bool(
                        settings_ctx,
                        runtime,
                        weak,
                        "offline-mode-enabled".to_string(),
                        value,
                    )
                    .await;
                });
            });
    }

    // B9 — offline Favorites "playable favorites" rail: rebuild on every
    // mount of the Favorites offline placeholder (the rail's init fires
    // load), play the rail from the clicked row.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<OfflineFavoritesActions>().on_load(move || {
            offline_favorites::load(weak.clone(), handle.clone());
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<OfflineFavoritesActions>().on_play(move |id| {
            offline_favorites::play(
                runtime.clone(),
                weak.clone(),
                handle.clone(),
                id.to_string(),
            );
        });
    }

    // Appearance settings persistence. The toggles/selects set their
    // AppearanceState property locally, then fire these generic callbacks so
    // the choice survives restart. Tray keys persist to the shared per-user
    // tray_settings store; unknown keys are logged (other appearance settings
    // are wired as they land).
    {
        let appearance = window.global::<AppearanceState>();
        let chrome_weak = window.as_weak();
        appearance.on_appearance_bool(move |key, value| match key.as_str() {
            "use-system-title-bar" => {
                // The toggle only edits the PREF (`use-system-title-bar`);
                // whether it reaches the applied chrome state
                // (`system-title-bar-active`, which no-frame / the header
                // drag+inset read) is decided HERE, per platform.
                let mut prefs = crate::ui_prefs::load();
                prefs.use_system_title_bar = value;
                crate::ui_prefs::save(&prefs);
                // Linux: mirror live (today's hot path — no-frame drives
                // winit set_decorations on the next properties update, and
                // the drawn controls/drag follow).
                #[cfg(not(target_os = "macos"))]
                if let Some(w) = chrome_weak.upgrade() {
                    w.global::<AppearanceState>()
                        .set_system_title_bar_active(value);
                }
                // macOS: persist-only, restart to apply. The overlay
                // attributes (titlebar_transparent / fullsize_content_view)
                // are fixed at window creation; flipping the header bindings
                // live desyncs them from the real chrome (traffic lights
                // overlapped by content, or system bar + overlay inset at
                // once), so `system-title-bar-active` keeps its startup
                // value there.
                crate::toast::info_weak(
                    &chrome_weak,
                    qbz_i18n::t("Title bar mode changed — restart QBZ to apply"),
                );
            }
            "hide-title-bar" => {
                // Live for the controls/drag (bindings read the flag); the
                // frameless state itself already follows use-system-title-bar.
                let mut prefs = crate::ui_prefs::load();
                prefs.hide_title_bar = value;
                crate::ui_prefs::save(&prefs);
            }
            "show-window-controls" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.show_window_controls = value;
                crate::ui_prefs::save(&prefs);
            }
            "album-header-gradient" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.album_header_gradient = value;
                crate::ui_prefs::save(&prefs);
            }
            "intelligent-search" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.intelligent_search = value;
                crate::ui_prefs::save(&prefs);
                // Propagate the toggle to the bound SearchService kill switch
                // (no-op if no session is bound; the next session init re-seeds
                // the flag from the persisted pref above).
                crate::search_service::set_enabled(value);
            }
            "system-notifications" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.system_notifications = value;
                crate::ui_prefs::save(&prefs);
                // Live gate for the poll-thread notify path (no restart).
                playback::NOTIFICATIONS_ENABLED
                    .store(value, std::sync::atomic::Ordering::Relaxed);
            }
            "tray-enable" => tray_settings::set_enable_tray(value),
            "tray-minimize-to-tray" => tray_settings::set_minimize_to_tray(value),
            "tray-close-to-tray" => tray_settings::set_close_to_tray(value),
            "tray-mac-hide-dock" => tray_settings::set_mac_hide_dock(value),
            "window-title-show" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.window_title_show = value;
                crate::ui_prefs::save(&prefs);
            }
            "show-volume-steppers" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.show_volume_steppers = value;
                crate::ui_prefs::save(&prefs);
            }
            "sidebar-playlist-collage" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.sidebar_playlist_collage = value;
                crate::ui_prefs::save(&prefs);
            }
            "local-library-track-artwork" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.local_library_track_artwork = value;
                crate::ui_prefs::save(&prefs);
            }
            "in-app-toasts" => {
                let mut prefs = crate::ui_prefs::load();
                prefs.in_app_toasts = value;
                crate::ui_prefs::save(&prefs);
            }
            other => log::debug!("[qbz-slint] unhandled appearance-bool '{other}'"),
        });
        let theme_weak = window.as_weak();
        let theme_handle = tokio::runtime::Handle::current();
        appearance.on_appearance_select(move |key, index| match key.as_str() {
            "tray-icon-theme" => {
                tray_settings::set_icon_theme_index(index);
                // Re-theme the running tray icon live (no restart).
                if let Some(t) = tray::handle() {
                    t.set_icon_theme(tray_settings::theme_for_index(index));
                }
            }
            "wc-position" => {
                // 0 = Left, 1 = Right. Live — HeaderBar re-anchors the drawn
                // controls from `wc-position-index`; persist only.
                let mut prefs = crate::ui_prefs::load();
                prefs.wc_position = if index == 0 { "left" } else { "right" }.to_string();
                crate::ui_prefs::save(&prefs);
            }
            "app-background" => {
                // 0 = Off, 1 = Ambient (GPU shader), 2 = Blurred art. The Slint
                // side already flipped app-background-mode-index; app-shader-mode
                // and app-background-active derive from it, and the AppShell
                // viz-should-run recompute starts/stops the drain reactively.
                let mut prefs = crate::ui_prefs::load();
                prefs.app_background =
                    crate::ui_prefs::app_background_for_index(index).to_string();
                crate::ui_prefs::save(&prefs);
            }
            "startup-page" => {
                // 0 = Home, 1 = Where you left off (restore last_view).
                let mut prefs = crate::ui_prefs::load();
                prefs.startup_page = crate::ui_prefs::startup_page_for_index(index).to_string();
                crate::ui_prefs::save(&prefs);
            }
            "renderer" => {
                // 0 = Auto, 1 = GPU (wgpu), 2 = GPU compatibility (femtovg GL),
                // 3 = Software. Startup-time choice — select_slint_backend()
                // reads it before the window exists, so it applies on the next
                // launch (a non-auto pick is protected by the auto-revert
                // sentinel there).
                let mut prefs = crate::ui_prefs::load();
                prefs.renderer = crate::ui_prefs::renderer_for_index(index).to_string();
                // A manual pick is the USER's choice — drop the auto-degrade
                // and alt-adapter markers so the ladder starts clean if they
                // ever return to "auto".
                prefs.renderer_auto_degraded.clear();
                prefs.renderer_wgpu_alt.clear();
                crate::ui_prefs::save(&prefs);
                crate::toast::info_weak(
                    &theme_weak,
                    qbz_i18n::t("Renderer changed — restart QBZ to apply"),
                );
            }
            "gpu-power" => {
                // 0 = Auto, i>0 = a specific detected GPU (by name). Applied at
                // startup by gpu_power_from_prefs (wgpu adapter power preference),
                // so it takes effect on the next launch.
                let mut prefs = crate::ui_prefs::load();
                prefs.gpu_power = gpu_power_value_for_index(index);
                crate::ui_prefs::save(&prefs);
                crate::toast::info_weak(
                    &theme_weak,
                    qbz_i18n::t("Preferred GPU changed — restart QBZ to apply"),
                );
            }
            "ui-scale" => {
                // 0 = Extra small, 1 = Small, 2 = Default, 3 = Large,
                // 4 = Extra large. Startup-time choice — SLINT_SCALE_FACTOR is
                // set at the very top of main() before the backend exists, so
                // it applies on the next launch.
                let mut prefs = crate::ui_prefs::load();
                prefs.ui_scale = crate::ui_prefs::ui_scale_for_index(index).to_string();
                crate::ui_prefs::save(&prefs);
                crate::toast::info_weak(
                    &theme_weak,
                    qbz_i18n::t("Interface size changed — restart QBZ to apply"),
                );
            }
            "language" => {
                // 0 = Auto, 1 = English, 2 = Español, 3 = Français, 4 = Deutsch,
                // 5 = Português, 6 = Русский, 7 = 日本語, 8 = Nederlands.
                // Persist the RAW user choice ("auto" stays "auto"), but resolve
                // "auto" to a concrete language before switching the live
                // translations.
                let chosen = crate::ui_prefs::language_for_index(index);
                let mut prefs = crate::ui_prefs::load();
                prefs.language = chosen.to_string();
                crate::ui_prefs::save(&prefs);
                let lang = if chosen == "auto" {
                    qbz_i18n::resolve_auto()
                } else {
                    chosen
                };
                qbz_i18n::set_language(lang);
                if let Err(e) = slint::select_bundled_translation(lang) {
                    log::warn!(
                        "[qbz-slint] select_bundled_translation('{lang}') failed: {e:?}"
                    );
                }
                // Reseed the non-reactive option arrays (they live as @tr
                // property DEFAULTS, which do NOT react to a translation
                // switch) so the dropdown contents update live.
                if let Some(w) = theme_weak.upgrade() {
                    reseed_i18n_labels(&w);
                }
            }
            "theme" => {
                // Slug is the source of truth. The appended "Auto (dynamic)"
                // entry (index == theme::auto_index) persists the "auto" slug and
                // generates the palette off-thread; every other index maps to a
                // stable registry id and hot-switches the static palette.
                let mut prefs = crate::ui_prefs::load();
                // The dropdown index is a position in the CURRENTLY filtered list
                // (All/Dark/Light), so map it through the same filter. Auto/Custom
                // only exist in the All view (filtered_*_index is -1 otherwise).
                let filter = prefs.theme_filter;
                if index == crate::theme::filtered_auto_index(filter) {
                    prefs.theme = crate::theme::AUTO_SLUG.to_string();
                    crate::ui_prefs::save(&prefs);
                    if let Some(w) = theme_weak.upgrade() {
                        let st = w.global::<AppearanceState>();
                        st.set_theme_is_auto(true);
                        st.set_theme_is_custom(false);
                        st.set_theme_is_system(false);
                    }
                    crate::auto_theme::regenerate(theme_weak.clone(), theme_handle.clone());
                } else if index == crate::theme::filtered_custom_index(filter) {
                    // "Custom": persist the slug, derive from the persisted (or
                    // freshly seeded) custom base, and apply live. The editor
                    // swatches are seeded from the same base.
                    prefs.theme = crate::theme::CUSTOM_SLUG.to_string();
                    crate::ui_prefs::save(&prefs);
                    if let Some(w) = theme_weak.upgrade() {
                        let st = w.global::<AppearanceState>();
                        st.set_theme_is_auto(false);
                        st.set_theme_is_custom(true);
                        st.set_theme_is_system(false);
                        if crate::custom_theme::exists() {
                            crate::custom_theme::seed_state(&w);
                            crate::custom_theme::apply_startup(&w);
                        } else {
                            // First-ever selection: seed from the palette the
                            // user is looking at RIGHT NOW (the previously
                            // applied theme), not from a hardcoded default —
                            // "customize what I see" is the whole point.
                            crate::custom_theme::seed_from_current(&w);
                        }
                    }
                } else {
                    let id = crate::theme::filtered_id_for_index(index, filter);
                    prefs.theme = id.slug().to_string();
                    crate::ui_prefs::save(&prefs);
                    if let Some(w) = theme_weak.upgrade() {
                        let st = w.global::<AppearanceState>();
                        st.set_theme_is_auto(false);
                        st.set_theme_is_custom(false);
                        st.set_theme_is_system(id == qbz_theme::ThemeId::System);
                        crate::theme::apply_theme(&w, id);
                    }
                }
            }
            "auto-theme-source" => {
                // Auto-theme source dropdown (System / Wallpaper / Custom Image):
                // persist the key and regenerate from the new source.
                crate::auto_theme::set_source(index, theme_weak.clone(), theme_handle.clone());
            }
            "theme-filter" => {
                // Theme-list filter cycle (0 = All, 1 = Dark, 2 = Light): persist
                // the choice, then rebuild the dropdown to show only matching
                // themes and re-derive which row (if any) is the active theme. The
                // applied palette is untouched — this only narrows the list.
                let mut prefs = crate::ui_prefs::load();
                prefs.theme_filter = index;
                crate::ui_prefs::save(&prefs);
                if let Some(w) = theme_weak.upgrade() {
                    let st = w.global::<AppearanceState>();
                    let labels: Vec<slint::SharedString> =
                        crate::theme::filtered_dropdown_labels(index)
                            .into_iter()
                            .map(slint::SharedString::from)
                            .collect();
                    st.set_themes(slint::ModelRc::new(slint::VecModel::from(labels)));
                    st.set_theme_index(crate::theme::filtered_selected_index_for_slug(
                        &prefs.theme,
                        index,
                    ));
                }
            }
            other => log::debug!("[qbz-slint] unhandled appearance-select '{other}'"),
        });

        // Auto-theme actions: image picker + explicit Regenerate button. Both run
        // generation off the event loop and push the palette back on it.
        let action_weak = window.as_weak();
        let action_handle = tokio::runtime::Handle::current();
        appearance.on_appearance_action(move |key| match key.as_str() {
            "auto-theme-select-image" => {
                crate::auto_theme::select_image(action_weak.clone(), action_handle.clone());
            }
            "auto-theme-regenerate" => {
                crate::auto_theme::regenerate(action_weak.clone(), action_handle.clone());
            }
            other => log::debug!("[qbz-slint] unhandled appearance-action '{other}'"),
        });

        // Custom-theme editor callbacks: per-token live edits (drag + hex),
        // polarity toggle, and "start from current theme". Each re-derives the
        // whole palette in Rust and pushes it live (derivation is cheap).
        let ct_weak = window.as_weak();
        appearance.on_custom_set_token(move |key, color| {
            if let Some(w) = ct_weak.upgrade() {
                crate::custom_theme::set_token(&w, key.as_str(), color);
            }
        });
        let ct_hex_weak = window.as_weak();
        appearance.on_custom_set_token_hex(move |key, hex| {
            if let Some(w) = ct_hex_weak.upgrade() {
                crate::custom_theme::set_token_hex(&w, key.as_str(), hex.as_str());
            }
        });
        let ct_dark_weak = window.as_weak();
        appearance.on_custom_toggle_dark(move |is_dark| {
            if let Some(w) = ct_dark_weak.upgrade() {
                crate::custom_theme::toggle_dark(&w, is_dark);
            }
        });
        let ct_seed_weak = window.as_weak();
        appearance.on_custom_seed_from_current(move || {
            if let Some(w) = ct_seed_weak.upgrade() {
                crate::custom_theme::seed_from_current(&w);
            }
        });
    }

    // "My QBZ" nav branding (Settings > Appearance) — persist the label /
    // custom icon per-user and re-seed MyQbzBrandingState so the sidebar row
    // updates live. Re-homed from the Tauri sidebar context-menu modal (DQ3).
    {
        let branding = window.global::<MyQbzBrandingState>();
        // Label: persist (blank coerces to "My QBZ" in the store) and push the
        // coerced value onto the shared `label` property so the sidebar row
        // updates live. We set only `label` (not a full re-seed) so the bound
        // LineEdit isn't disturbed mid-edit beyond the documented blank->default
        // coercion. The icon state is left untouched here.
        let weak = window.as_weak();
        branding.on_set_label(move |label| {
            let coerced = myqbz_prefs::set_label(label.as_str());
            if let Some(w) = weak.upgrade() {
                w.global::<MyQbzBrandingState>().set_label(coerced.into());
            }
        });
        // Change icon: async native picker; persists + re-seeds on pick.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        branding.on_pick_icon(move || {
            myqbz_prefs::pick_icon(weak.clone(), handle.clone());
        });
        // Reset icon: clear the custom path, re-seed to the default glyph.
        let weak = window.as_weak();
        branding.on_reset_icon(move || {
            myqbz_prefs::reset_icon();
            if let Some(w) = weak.upgrade() {
                myqbz_prefs::seed(&w);
            }
        });
    }

    // Pin / unpin from the card pin glyphs. The callback carries the full
    // display snapshot (kind, id, title, subtitle, artwork url) so the store
    // persists a denormalized row without re-fetching (the
    // BlacklistActions.block-album pattern). The mutation is local SQLite
    // (synchronous): mutate first, then — only on success — flip the
    // `is-pinned` badge across every visible card model and rebuild the
    // Pinned section from the store (the ONE rebuild path: model first,
    // then a fresh index-keyed artwork job batch).
    {
        let weak = window.as_weak();
        window
            .global::<PinnedActions>()
            .on_toggle_pin(move |kind, id, title, subtitle, artwork| {
                if let Some(w) = weak.upgrade() {
                    let kind = kind.to_string();
                    let id = id.to_string();
                    // The cards hardcode these kinds and the store's CHECK
                    // constraint admits nothing else — anything different is
                    // a wiring bug.
                    if !matches!(kind.as_str(), "album" | "artist" | "playlist") {
                        log::warn!("[qbz-slint] toggle-pin: unsupported kind {kind}");
                        return;
                    }
                    let was_pinned = crate::pinned::is_pinned(&kind, &id);
                    let res = if was_pinned {
                        crate::pinned::unpin(&kind, &id)
                    } else {
                        crate::pinned::pin(&crate::pinned::PinnedItem {
                            kind: kind.clone(),
                            id: id.clone(),
                            title: title.to_string(),
                            subtitle: subtitle.to_string(),
                            artwork_url: artwork.to_string(),
                            pinned_at: 0, // ignored on write; the service stamps now
                        })
                    };
                    match res {
                        Ok(()) => {
                            let pinned = !was_pinned;
                            // Flip the card badges AND the open detail view's
                            // header pin (when it is showing this same id).
                            match kind.as_str() {
                                "album" => {
                                    set_album_row_pinned(&w, &id, pinned);
                                    let st = w.global::<AlbumState>();
                                    if st.get_id().as_str() == id {
                                        st.set_pinned(pinned);
                                    }
                                }
                                "artist" => {
                                    set_artist_row_pinned(&w, &id, pinned);
                                    let st = w.global::<ArtistState>();
                                    if st.get_id().as_str() == id {
                                        st.set_pinned(pinned);
                                    }
                                }
                                "playlist" => {
                                    set_playlist_row_pinned(&w, &id, pinned);
                                    let st = w.global::<PlaylistState>();
                                    if st.get_id().as_str() == id {
                                        st.set_pinned(pinned);
                                    }
                                }
                                _ => {}
                            }
                            crate::pinned_section::rebuild_pinned(&w);
                        }
                        Err(e) => {
                            // Local store mutation failed (no session / DB
                            // error): nothing was flipped, so there is nothing
                            // to revert — surface the sibling stores' message.
                            log::error!("[qbz-slint] toggle-pin {kind} {id} failed: {e}");
                            crate::toast::error(&w, e);
                        }
                    }
                }
            });
    }

    // Context-menu / overlay media actions — route play / queue actions
    // into the playback controller; favorite / download stay logged.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_media_action(move |kind, id, action| {
            let kind = kind.to_string();
            let id = id.to_string();
            let action = action.to_string();
            log::info!("[qbz-slint] media-action: kind={kind} id={id} action={action}");
            // Local Library album detail reuses AlbumPageView. Route its play
            // actions to local playback — guarded to the album view + is-local
            // so Qobuz album/track play is untouched.
            if action == "play" && (kind == "album" || kind == "track") {
                if let Some(w) = weak.upgrade() {
                    let album_state = w.global::<AlbumState>();
                    if matches!(w.global::<NavState>().get_view(), ContentView::Album)
                        && album_state.get_is_local()
                    {
                        let album_id = album_state.get_id().to_string();
                        let start = if kind == "track" {
                            id.parse::<i64>().ok()
                        } else {
                            None
                        };
                        playback::play_local_album(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            album_id,
                            start,
                        );
                        return;
                    }
                }
            }

            // === Capa B feedback (intelligent search) ====================
            // Feed the ranking model from RESULTS-PAGE clicks, gated to the
            // Search view inside `record_search_interaction` so the same global
            // media-action handler fired from other views never mis-attributes.
            // Only QOBUZ result clicks are recorded; the search results page
            // never carries local rows (D1/D2), so no source check is needed.
            //   - track play              -> Play
            //   - album play              -> Play (an album-card play is still a
            //                                play interaction with the entity)
            //   - album favorite (toggle) -> Favorite ONLY when transitioning to
            //                                favorited (the card heart arm is a
            //                                toggle since 2026-07; Favorite
            //                                weight must only ADD)
            //   - artist follow (add)     -> Favorite (search artist cards show
            //                                "Follow" only when NOT following, so
            //                                this action is always an add)
            //   - track favorite (toggle) -> Favorite ONLY when transitioning to
            //                                favorited (Favorite weight must only
            //                                ADD — never record on un-favorite)
            if let Some(w) = weak.upgrade() {
                use crate::search_service::InteractionAction;
                match (kind.as_str(), action.as_str()) {
                    ("track", "play") | ("album", "play") => {
                        record_search_interaction(&w, &kind, &id, InteractionAction::Play);
                    }
                    ("album", "favorite") => {
                        // Toggle: record ONLY when this click ADDS the favorite
                        // (mirrors the track arm below; the album card arm flips
                        // off the same `fav_cache::is_album_favorite`).
                        if !crate::fav_cache::is_album_favorite(&id) {
                            record_search_interaction(&w, &kind, &id, InteractionAction::Favorite);
                        }
                    }
                    ("artist", "follow") => {
                        // Add-only on a search card ("Follow" shows only when
                        // NOT following).
                        record_search_interaction(&w, &kind, &id, InteractionAction::Favorite);
                    }
                    ("track", "favorite") => {
                        // Toggle: record ONLY when this click ADDS the favorite
                        // (the current cached state is "not favorite"). Reading
                        // the pre-toggle state here matches `toggle_track_favorite`,
                        // which flips off the same `fav_cache::is_favorite`.
                        if !crate::fav_cache::is_favorite(&id) {
                            record_search_interaction(&w, &kind, &id, InteractionAction::Favorite);
                        }
                    }
                    _ => {}
                }
            }

            match (kind.as_str(), action.as_str()) {
                // Large dock: visualizer on/off toggle (the cover's eye button).
                // Routed through Rust so the choice persists in ui_prefs; the
                // AppShell viz-should-run handler idles the FFT tap when off.
                ("npb-large", "viz-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let shell = w.global::<ShellState>();
                        let on = !shell.get_large_visualizer_on();
                        shell.set_large_visualizer_on(on);
                        let mut prefs = crate::ui_prefs::load();
                        prefs.large_visualizer = on;
                        crate::ui_prefs::save(&prefs);
                    }
                }
                // Large dock: cycle the spectrum visualization (Bars -> Waveform
                // -> Energy), persisted in ui_prefs.
                ("npb-large", "spectrum-cycle") => {
                    if let Some(w) = weak.upgrade() {
                        let shell = w.global::<ShellState>();
                        let next = (shell.get_large_spectrum_mode() + 1).rem_euclid(3);
                        shell.set_large_spectrum_mode(next);
                        let mut prefs = crate::ui_prefs::load();
                        prefs.large_spectrum_mode =
                            crate::ui_prefs::large_spectrum_mode_key(next).to_string();
                        crate::ui_prefs::save(&prefs);
                    }
                }
                // Track Info modal — opened from the NPB (i) button, the
                // song-card title, or a TrackRow context menu. Qobuz tracks
                // only (the id must be a real catalog u64).
                ("track", "track-info") => {
                    if let Ok(track_id) = id.parse::<u64>() {
                        info_modals::open_track_info(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                // "Reveal in file explorer" — local tracks only (the row's
                // id is a library row id, not a catalog id; TrackContextMenu
                // gates the menu entry itself on source == "local").
                // Try the in-memory Tracks-tab cache first (no DB hit);
                // folder-detail rows that aren't in it fall back to an
                // off-thread DB resolve, mirroring the play-next/queue arm
                // just above this match's local block.
                ("track", "reveal-in-explorer") => {
                    if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                        reveal_in_file_manager(&row.file_path);
                    } else if let Ok(rid) = id.parse::<i64>() {
                        handle.spawn(async move {
                            let row = tokio::task::spawn_blocking(move || {
                                crate::library_db::with_db(|db| db.get_track(rid)).flatten()
                            })
                            .await
                            .ok()
                            .flatten();
                            if let Some(row) = row {
                                reveal_in_file_manager(&row.file_path);
                            }
                        });
                    }
                }
                // Album Info (Credits/Review) modal — opened from the album
                // header (i) button. Qobuz albums only (skip local keys).
                ("album", "info") => {
                    if !is_local_album_key(&id) {
                        info_modals::open_album_credits(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                        );
                    }
                }
                // Album booklet (digital liner-notes PDF) — the album-header
                // booklet button DOWNLOADS the goody PDF (stashed by
                // album::apply_album) to a user-chosen location. No-op when the
                // album bundles no booklet (empty stashed URL).
                ("album", "booklet") => {
                    crate::booklet::download_booklet(weak.clone(), handle.clone());
                }
                // "From the same artist" carousel "View all" — open the artist's
                // full Albums discography page. `id` is the artist id; reuse the
                // dedicated releases page (release_type "album").
                ("artist", "releases") => {
                    if !id.is_empty() {
                        let name = weak
                            .upgrade()
                            .map(|w| w.global::<AlbumState>().get_artist().to_string())
                            .unwrap_or_default();
                        nav::record(nav::NavEntry::ArtistReleases {
                            id: id.clone(),
                            name: name.clone(),
                            release_type: "album".to_string(),
                        });
                        navigate_artist_releases(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            id.clone(),
                            name,
                            "album".to_string(),
                        );
                        if let Some(w) = weak.upgrade() {
                            update_nav_flags(&w);
                        }
                    }
                }
                ("album", "play") => {
                    // A local id is a metadata group key, not a Qobuz id —
                    // play it from the local cache (Home "Recently played",
                    // etc.) instead of trying to fetch a Qobuz album.
                    if is_local_album_key(&id) {
                        playback::play_local_album(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                            None,
                        );
                    } else {
                        playback::play_album(runtime.clone(), weak.clone(), handle.clone(), id, 0);
                    }
                }
                ("track", "play") => {
                    // Universal per-row play: queue the current view's VISIBLE
                    // tracklist starting at the clicked track (see
                    // playback::play_track_in_context). Every tracklist surface
                    // routes here — album, playlist, favorites, label, mix,
                    // artist, search.
                    if let Some(w) = weak.upgrade() {
                        playback::play_track_in_context(
                            &w,
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            &id,
                        );
                    }
                }
                ("album", "queue") => playback::enqueue_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("track", "queue") => {
                    // SOURCE-TYPED routing first (spec §3.2, mirrors the
                    // add-to-playlist arm): on a snapshot-backed playlist
                    // detail a local row's id is a library row id — the
                    // catalog path below would mis-resolve it (wrong-track
                    // hazard / silent failure). The merged snapshot carries
                    // the ready, source-aware QueueTrack; enqueue it directly.
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    playback::enqueue_queue_tracks(
                                        runtime.clone(),
                                        weak.clone(),
                                        handle.clone(),
                                        vec![qt],
                                        false,
                                    );
                                    return;
                                }
                            }
                        }
                    }
                    // Qobuz rows (incl. offline copies with real catalog
                    // ids): the existing path — single-track
                    // admission + fresh fetch.
                    if let Ok(track_id) = id.parse::<u64>() {
                        playback::enqueue_track(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("album", "play-next") => playback::enqueue_album_next(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("album", "shuffle") => playback::play_album_shuffled(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("album", "edit") => {
                    // Open the local-album tag editor (group_key == directory_path
                    // for folder-grouped local albums).
                    tag_editor::open_tag_editor(weak.clone(), handle.clone(), id.clone(), id);
                }
                ("album", "add-to-mixtape") => {
                    // The cassette button on the album header. Local albums
                    // build the payload
                    // from AlbumState + the loaded tracks; Qobuz albums resolve
                    // via get_album (the proven fail-safe resolver).
                    let Some(w) = weak.upgrade() else { return };
                    let st = w.global::<AlbumState>();
                    if st.get_is_local() {
                        let item = myqbz_add::AddItem {
                            item_type: "album".into(),
                            source: "local".into(),
                            source_item_id: st.get_id().to_string(),
                            title: st.get_title().to_string(),
                            subtitle: {
                                let a = st.get_artist().to_string();
                                (!a.is_empty()).then_some(a)
                            },
                            artwork_url: None, // local albums omit artwork_url (1:1 PSD)
                            year: None,
                            track_count: {
                                use slint::Model;
                                let n = st.get_tracks().row_count();
                                (n > 0).then_some(n as i32)
                            },
                        };
                        open_add_to_mixtape(weak.clone(), handle.clone(), vec![item]);
                    } else {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let handle2 = handle.clone();
                        let album_id = id.clone();
                        handle.spawn(async move {
                            let item = match runtime.core().get_album(&album_id).await {
                                Ok(album) => {
                                    let artwork_url = album
                                        .image
                                        .thumbnail
                                        .clone()
                                        .or_else(|| album.image.small.clone());
                                    let year = album
                                        .release_date_original
                                        .as_deref()
                                        .and_then(|d| d.get(0..4))
                                        .and_then(|y| y.parse::<i32>().ok());
                                    let track_count = album
                                        .tracks_count
                                        .or(album.track_count)
                                        .map(|n| n as i32);
                                    myqbz_add::AddItem {
                                        item_type: "album".into(),
                                        source: "qobuz".into(),
                                        source_item_id: album.id.clone(),
                                        title: album.title.clone(),
                                        subtitle: {
                                            let a = album.artist.name.clone();
                                            (!a.is_empty()).then_some(a)
                                        },
                                        artwork_url,
                                        year,
                                        track_count,
                                    }
                                }
                                Err(e) => {
                                    log::warn!(
                                        "[qbz-slint] add-to-mixtape: get_album {album_id} failed: {e}"
                                    );
                                    return;
                                }
                            };
                            open_add_to_mixtape(weak, handle2, vec![item]);
                        });
                    }
                }
                ("album", "favorite") => {
                    // Album-card heart + "…" menu entry: a TRUE TOGGLE keyed
                    // off the favorite-album cache (filled heart → remove,
                    // empty → add), mirroring the header "favorite-toggle"
                    // arm below. Was add-only while the cards couldn't show
                    // favorite state; now that they do, re-adding from a
                    // filled heart would lie. Optimistic: flip the heart on
                    // every visible card right away (mirrors the track
                    // rows); rolled back on failure. NOTE: the Favorites
                    // albums tab never reaches this arm — FavoritesView
                    // intercepts "favorite" to unfavorite-album (fade-out +
                    // row removal).
                    let was_fav = crate::fav_cache::is_album_favorite(&id);
                    let new_state = !was_fav;
                    if let Some(w) = weak.upgrade() {
                        set_album_row_favorite(&w, &id, new_state);
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let album_id = id.clone();
                    handle.spawn(async move {
                        let res = if new_state {
                            runtime.core().add_favorite("album", &album_id).await
                        } else {
                            runtime.core().remove_favorite("album", &album_id).await
                        };
                        match res {
                            Ok(()) => {
                                // Keep the favorite-album cache in sync so the
                                // album-header heart reflects a card toggle.
                                crate::fav_cache::set_album(&album_id, new_state);
                                crate::toast::success_weak(
                                    &weak,
                                    if new_state {
                                        "Added to favorites"
                                    } else {
                                        "Removed from favorites"
                                    },
                                );
                                // reco: log the album favorite ADD on success
                                // only — Capa B scores adds, never removals.
                                if new_state {
                                    let aid = album_id.clone();
                                    tokio::task::spawn_blocking(move || {
                                        crate::reco::log_favorite_album(aid, None)
                                    });
                                }
                            }
                            Err(e) => {
                                log::error!(
                                    "[qbz-slint] toggle favorite album {album_id} failed: {e}"
                                );
                                crate::toast::error_weak(&weak, "Couldn't update favorites");
                                // Roll the optimistic hearts back to the
                                // pre-click state.
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    set_album_row_favorite(&w, &album_id, was_fav);
                                });
                            }
                        }
                    });
                }
                ("album", "favorite-toggle") => {
                    // The album-header heart: a TRUE toggle that reflects the
                    // favorite-album cache (the card "favorite" arm above is
                    // the same toggle, minus the AlbumState header sync).
                    // Optimistic on the open header, reconciled on the server
                    // result.
                    let Some(w) = weak.upgrade() else {
                        return;
                    };
                    let was_fav = crate::fav_cache::is_album_favorite(&id);
                    let new_state = !was_fav;
                    let st = w.global::<AlbumState>();
                    let is_open = st.get_id() == id.as_str();
                    if is_open {
                        st.set_is_favorite(new_state);
                        st.set_favorite_loading(true);
                    }
                    // Optimistic on every visible album card too (artist
                    // discography, carousels, search, favorites) — reconciled
                    // with the server result below, like the header heart.
                    set_album_row_favorite(&w, &id, new_state);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let album_id = id.clone();
                    handle.spawn(async move {
                        let res = if new_state {
                            runtime.core().add_favorite("album", &album_id).await
                        } else {
                            runtime.core().remove_favorite("album", &album_id).await
                        };
                        let ok = res.is_ok();
                        if let Err(e) = &res {
                            log::error!(
                                "[qbz-slint] toggle favorite album {album_id} failed: {e}"
                            );
                        }
                        // reco: log the album favorite ADD on success (skip the
                        // un-favorite). Blocking SQLite off the async path.
                        if ok && new_state {
                            let aid = album_id.clone();
                            tokio::task::spawn_blocking(move || {
                                crate::reco::log_favorite_album(aid, None)
                            });
                        }
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            let st = w.global::<AlbumState>();
                            let open_now = st.get_id() == album_id.as_str();
                            if ok {
                                crate::fav_cache::set_album(&album_id, new_state);
                                if open_now {
                                    st.set_favorite_loading(false);
                                    st.set_is_favorite(new_state);
                                }
                                crate::toast::success(
                                    &w,
                                    if new_state {
                                        "Added to favorites"
                                    } else {
                                        "Removed from favorites"
                                    },
                                );
                            } else {
                                if open_now {
                                    st.set_favorite_loading(false);
                                    st.set_is_favorite(was_fav);
                                }
                                // Roll the optimistic card hearts back too.
                                set_album_row_favorite(&w, &album_id, was_fav);
                                crate::toast::error(&w, "Couldn't update favorites");
                            }
                        });
                    });
                }
                ("album", "cache") => offline_cache::cache_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                ),
                ("album", "recache") => offline_cache::redownload_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                    // Refresh the WHOLE album (Tauri's "Refresh offline copy"
                    // re-downloads every track, not only the failed ones).
                    false,
                ),
                ("album", "add-to-playlist") => {
                    // Resolve the album's loaded tracks to their Qobuz catalog
                    // ids and open the playlist picker for the whole set
                    // (mirrors Tauri's album → Add to playlist). Local
                    // albums carry no catalog ids, so the entry no-ops there
                    // (the header menu is a Qobuz surface).
                    let Some(w) = weak.upgrade() else {
                        return;
                    };
                    let ids: Vec<String> = {
                        use slint::Model;
                        w.global::<AlbumState>()
                            .get_tracks()
                            .iter()
                            .map(|t| t.id.to_string())
                            .filter(|s| s.parse::<u64>().is_ok())
                            .collect()
                    };
                    if ids.is_empty() {
                        toast::error(&w, "No tracks to add");
                        return;
                    }
                    playlist_picker::open_multi(&w, &ids, false);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        let playlists = playlist_picker::load(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, playlists);
                        });
                    });
                }
                ("album", "share-qobuz") => {
                    share::copy_to_clipboard(share::qobuz_album_url(&id));
                    log::info!("[qbz-slint] copied Qobuz link for album {id}");
                }
                ("album", "share-songlink") => {
                    // Tauri-parity resolution (#514): fetch the album to get
                    // its UPC, then UPC -> Deezer -> album.link. The old
                    // URL-only Odesli call never worked for Qobuz input
                    // (could_not_resolve_entity) — see share.rs.
                    let album = id.clone();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    crate::toast::info_weak(&weak, qbz_i18n::t("Fetching Album.link..."));
                    handle.spawn(async move {
                        let upc = runtime
                            .core()
                            .get_album(&album)
                            .await
                            .ok()
                            .and_then(|a| a.upc);
                        match share::albumlink_for_album(&album, upc.as_deref()).await {
                            Some(url) => {
                                share::copy_to_clipboard(url);
                                log::info!("[qbz-slint] copied Album.link for album {album}");
                                crate::toast::success_weak(&weak, qbz_i18n::t("Link copied"));
                            }
                            None => {
                                log::warn!("[qbz-slint] Album.link resolution failed for {album}");
                                crate::toast::error_weak(
                                    &weak,
                                    qbz_i18n::t("Failed to copy link"),
                                );
                            }
                        }
                    });
                }
                ("track", "play-next") => {
                    // Source-typed routing — see the ("track","queue") arm
                    // (same seam, insert-next instead of append).
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    playback::enqueue_queue_tracks(
                                        runtime.clone(),
                                        weak.clone(),
                                        handle.clone(),
                                        vec![qt],
                                        true,
                                    );
                                    return;
                                }
                            }
                        }
                    }
                    if let Ok(track_id) = id.parse::<u64>() {
                        playback::play_track_next(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "favorite") => {
                    // Offline guard + optimistic toggle + network flip with
                    // rollback — shared with the library-surface favorite
                    // (see toggle_track_favorite).
                    toggle_track_favorite(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id.to_string(),
                    );
                }
                // Offline cache: "download"/"cache" make a track available
                // offline; "uncache" removes the local copy. The row affordance
                // and the context menu both bubble these.
                ("track", "cache") | ("track", "download") => {
                    if let Ok(track_id) = id.parse::<u64>() {
                        offline_cache::cache_track(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "uncache") => {
                    if let Ok(track_id) = id.parse::<u64>() {
                        offline_cache::remove_cached(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "recache") => {
                    // "Refresh offline copy" (cached-state menu entry, spec
                    // §3.5): remove + re-download, sequenced.
                    if let Ok(track_id) = id.parse::<u64>() {
                        offline_cache::refresh_cached(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            track_id,
                        );
                    }
                }
                ("track", "remove-from-playlist") => {
                    // Per-row removal on the playlist detail (spec §3.1).
                    // Ownership-gated in the UI (PlaylistState.is-owner —
                    // DELIBERATE: Tauri's available branch renders it
                    // un-gated on followed playlists where the owner-only
                    // API rejects, §1.6.1; we port the intent, not the
                    // hole). One-row ride on the same namespace-split seam
                    // as the bulk removal; the reload re-merges the sidecar.
                    let Some(w) = weak.upgrade() else { return };
                    if w.global::<NavState>().get_view() != ContentView::Playlist {
                        return;
                    }
                    if w.global::<PlaylistState>().get_is_local() {
                        local_playlist::remove_rows_by_ids(
                            &w,
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            image_cache.clone(),
                            vec![id.to_string()],
                        );
                        return;
                    }
                    let pid = w.global::<PlaylistState>().get_id().to_string();
                    let Some(row) = playlist::row_for_id(&id) else {
                        log::warn!("[qbz-slint] remove-from-playlist: row {id} not loaded");
                        return;
                    };
                    if let Ok(pid) = pid.parse::<u64>() {
                        playlist_remove_rows(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            image_cache.clone(),
                            pid,
                            vec![row],
                        );
                    }
                }
                // External-reco Weekly rows (P7): the title-adjacent buttons.
                // `id` carries the section key ("weekly-exploration"/"weekly-jams").
                ("ext-reco-list", "queue") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = external_reco::list_track_ids(&w, &id);
                        playback::enqueue_track_ids(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            ids,
                            false,
                        );
                    }
                }
                ("ext-reco-list", "create-playlist") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = external_reco::list_track_ids(&w, &id);
                        if !ids.is_empty() {
                            let ids_str: Vec<String> =
                                ids.iter().map(|i| i.to_string()).collect();
                            playlist_picker::open_for_ids(
                                &w,
                                runtime.clone(),
                                &handle,
                                ids_str,
                                false,
                            );
                        }
                    }
                }
                ("track", "add-to-playlist") => {
                    // Open the global picker for this track + load the
                    // user's playlists. SOURCE-TYPED routing first: this
                    // shared arm also fires for local rows (local
                    // playlist detail, now-playing), whose ids are NOT
                    // Qobuz catalog ids. Type the ref, or refuse.
                    let Some(w) = weak.upgrade() else {
                        return;
                    };
                    // Only consult the local-playlist queue snapshot while
                    // its detail is the OPEN view — a stale snapshot row id
                    // could collide with a genuine catalog id from a Qobuz
                    // surface (both are small integers). The ONLINE mixed
                    // Qobuz detail shares the snapshot (E11), so its
                    // local rows type their refs the same way.
                    let in_local_detail = snapshot_detail_open(&w);
                    let local_ref: Option<String> = if in_local_detail {
                        // Open local-playlist detail row: the queue snapshot
                        // knows its source ("<row id>"; None for Qobuz rows
                        // = catalog flow below).
                        local_playlist::local_picker_ref_for_row(id.as_str())
                    } else {
                        None
                    };
                    if let Some(track_ref) = local_ref {
                        playlist_picker::open_multi(&w, &[track_ref], true);
                    } else if id
                        .parse::<u64>()
                        .is_ok_and(|n| n >= local_library::LEGACY_SYNTHETIC_ID_FLOOR)
                    {
                        // A synthetic (ephemeral) id with no resolvable
                        // ref — refuse rather than store a fake Qobuz id.
                        log::warn!(
                            "[qbz-slint] add-to-playlist: unresolvable non-catalog id {id} — refused"
                        );
                        toast::error(&w, "Couldn't resolve this track");
                        return;
                    } else {
                        playlist_picker::open(&w, &id);
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        let playlists = playlist_picker::load(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, playlists);
                        });
                    });
                }
                ("track", "add-to-mixtape") => {
                    // The menu only carries the track id; resolve the Qobuz
                    // track (this entry is gated to Qobuz/offline in the menu)
                    // to build the AddToMixtape payload, then open the picker.
                    if let Ok(track_id) = id.parse::<u64>() {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let handle2 = handle.clone();
                        handle.spawn(async move {
                            let item = match runtime.core().get_track(track_id).await {
                                Ok(track) => {
                                    let artist = track
                                        .performer
                                        .as_ref()
                                        .map(|p| p.name.clone())
                                        .unwrap_or_default();
                                    let album = track
                                        .album
                                        .as_ref()
                                        .map(|a| a.title.clone())
                                        .unwrap_or_default();
                                    let subtitle = [artist, album]
                                        .into_iter()
                                        .filter(|s| !s.is_empty())
                                        .collect::<Vec<_>>()
                                        .join(" · ");
                                    let artwork_url = track.album.as_ref().and_then(|a| {
                                        a.image
                                            .thumbnail
                                            .clone()
                                            .or_else(|| a.image.small.clone())
                                    });
                                    myqbz_add::AddItem {
                                        item_type: "track".into(),
                                        source: "qobuz".into(),
                                        source_item_id: track_id.to_string(),
                                        title: track.title.clone(),
                                        subtitle: (!subtitle.is_empty()).then_some(subtitle),
                                        artwork_url,
                                        year: None,
                                        track_count: None,
                                    }
                                }
                                Err(e) => {
                                    log::warn!(
                                        "[qbz-slint] add-to-mixtape: get_track {track_id} failed: {e}"
                                    );
                                    return;
                                }
                            };
                            open_add_to_mixtape(weak, handle2, vec![item]);
                        });
                    }
                }
                ("track", "share-qobuz") => {
                    share::copy_to_clipboard(share::qobuz_track_url(&id));
                    log::info!("[qbz-slint] copied Qobuz link for track {id}");
                }
                ("track", "share-songlink") => {
                    // Tauri-parity resolution (#514): fetch the track to get
                    // its ISRC, then ISRC -> Deezer -> song.link. The old
                    // URL-only Odesli call never worked for Qobuz input
                    // (could_not_resolve_entity) — see share.rs.
                    let track = id.clone();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    crate::toast::info_weak(&weak, qbz_i18n::t("Fetching Song.link..."));
                    handle.spawn(async move {
                        let isrc = match track.parse::<u64>() {
                            Ok(tid) => runtime
                                .core()
                                .get_track(tid)
                                .await
                                .ok()
                                .and_then(|t| t.isrc),
                            Err(_) => None,
                        };
                        match share::songlink_for_track(&track, isrc.as_deref()).await {
                            Some(url) => {
                                share::copy_to_clipboard(url);
                                log::info!("[qbz-slint] copied Song.link for track {track}");
                                crate::toast::success_weak(&weak, qbz_i18n::t("Link copied"));
                            }
                            None => {
                                log::warn!("[qbz-slint] Song.link resolution failed for {track}");
                                crate::toast::error_weak(
                                    &weak,
                                    qbz_i18n::t("Failed to copy link"),
                                );
                            }
                        }
                    });
                }
                ("track", "go-to-album") => {
                    // Playlist-detail local sidecar rows first (owner
                    // improvement — Tauri omits the entries there): their
                    // snapshot ids are library row ids, NOT catalog ids, and
                    // the snapshot QueueTrack's album_id already carries the
                    // LOCAL navigation key (the same one the now-playing bar
                    // navigates by — group key). Qobuz + offline-copy rows fall
                    // through to the catalog resolve below (an offline copy's
                    // row id IS its Qobuz id).
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    match qt.album_id.filter(|k| !k.is_empty()) {
                                        Some(key) => w.invoke_open_album(key.into()),
                                        None => log::debug!(
                                            "[qbz-slint] go-to-album: playlist row {id} has no album key"
                                        ),
                                    }
                                    return;
                                }
                            }
                        }
                    }
                    // The menu only carries the track id — resolve the
                    // track to find its album, then open it.
                    if let Ok(track_id) = id.parse::<u64>() {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            if let Ok(track) = runtime.core().get_track(track_id).await {
                                if let Some(album_id) =
                                    track.album.as_ref().map(|a| a.id.clone())
                                {
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        w.invoke_open_album(album_id.into());
                                    });
                                }
                            }
                        });
                    }
                }
                ("track", "go-to-artist") => {
                    // Same local diversion as go-to-album: local
                    // artists have no id, so route by NAME to the LocalLibrary
                    // Artists tab (the open-artist callback's split).
                    if let Some(w) = weak.upgrade() {
                        if snapshot_detail_open(&w) {
                            if let Some(qt) = local_playlist::queue_track_for_row(&id) {
                                if matches!(qt.source.as_deref(), Some("local")) {
                                    if qt.artist.trim().is_empty() {
                                        log::debug!(
                                            "[qbz-slint] go-to-artist: playlist row {id} has no artist name"
                                        );
                                    } else {
                                        w.invoke_open_artist(qt.artist.into());
                                    }
                                    return;
                                }
                            }
                        }
                    }
                    if let Ok(track_id) = id.parse::<u64>() {
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            if let Ok(track) = runtime.core().get_track(track_id).await {
                                if let Some(artist_id) =
                                    track.performer.as_ref().map(|p| p.id)
                                {
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        w.invoke_open_artist(artist_id.to_string().into());
                                    });
                                }
                            }
                        });
                    }
                }
                // Clickable artist name (album cards) -> artist page.
                ("artist", "open") => {
                    if let Some(w) = weak.upgrade() {
                        w.invoke_open_artist(id.clone().into());
                    }
                }
                // Clickable album name (track rows) -> album page.
                ("album", "open") => {
                    if let Some(w) = weak.upgrade() {
                        w.invoke_open_album(id.clone().into());
                    }
                }
                // Now-playing context (song-card layers button) -> playlist page.
                ("playlist", "open") => {
                    nav::record(nav::NavEntry::Playlist(id.clone()));
                    navigate_playlist(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        id.clone(),
                    );
                }
                // Blacklist / Show toggle from the ArtistView overflow
                // menu (and the hidden-artist banner). Resolves the id
                // from the passed value, falling back to ArtistState.id
                // Reads the name from
                // ArtistState for storage. Optimistic with rollback: flip
                // ArtistState.is-blacklisted immediately, perform the
                // mutation, revert + error-toast on failure. Synchronous
                // on the event-loop thread, so there is no re-entrancy
                // window (a second click can't interleave mid-toggle).
                ("artist", "share") => {
                    let artist_id = if id.is_empty() {
                        weak.upgrade()
                            .map(|w| w.global::<ArtistState>().get_id().to_string())
                            .unwrap_or_default()
                    } else {
                        id.clone()
                    };
                    if !artist_id.is_empty() {
                        share::copy_to_clipboard(share::qobuz_artist_url(&artist_id));
                        if let Some(w) = weak.upgrade() {
                            crate::toast::success(&w, qbz_i18n::t("Link copied"));
                        }
                    }
                }
                ("artist", "blacklist-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let st = w.global::<ArtistState>();
                        let artist_id = if id.is_empty() {
                            st.get_id().to_string()
                        } else {
                            id.clone()
                        };
                        let name = st.get_name().to_string();
                        if let Ok(id_num) = artist_id.parse::<u64>() {
                            let was_blacklisted =
                                crate::artist_blacklist::is_blacklisted(id_num);
                            // Optimistic flip.
                            st.set_is_blacklisted(!was_blacklisted);
                            let res = if was_blacklisted {
                                crate::artist_blacklist::remove(id_num)
                            } else {
                                crate::artist_blacklist::add(
                                    id_num,
                                    &name,
                                    None,
                                )
                            };
                            match res {
                                Ok(()) => {
                                    // Live refresh for the artist page is the
                                    // optimistic ArtistState.is-blacklisted
                                    // flip above (drives the banner + the
                                    // menu Show/Blacklist label). ArtistView
                                    // popular-tracks rows are deliberately
                                    // NOT per-row greyed (T6 scoping — the
                                    // banner is the artist-page surface);
                                    // other open views (search, album,
                                    // favorites) re-stamp on next navigation
                                    // (no global observer).
                                    let msg = if was_blacklisted {
                                        format!("{name} is now visible")
                                    } else {
                                        format!("{name} is now hidden")
                                    };
                                    crate::toast::success_weak(&weak, msg);
                                }
                                Err(e) => {
                                    log::error!(
                                        "[qbz-slint] blacklist toggle failed: {e}"
                                    );
                                    // Rollback the optimistic flip.
                                    st.set_is_blacklisted(was_blacklisted);
                                    crate::toast::error_weak(
                                        &weak,
                                        "Failed to update artist visibility",
                                    );
                                }
                            }
                        }
                    }
                }
                ("album", "block") | ("album", "unblock") => {
                    if let Some(w) = weak.upgrade() {
                        let st = w.global::<AlbumState>();
                        // Header menu: the open album is AlbumState, so resolve
                        // the display fields (title/artist/cover) from it.
                        let album_id = if id.is_empty() {
                            st.get_id().to_string()
                        } else {
                            id.clone()
                        };
                        if !album_id.is_empty() {
                            let was_blocked =
                                crate::artist_blacklist::is_album_blacklisted(&album_id);
                            // Optimistic flip on the header toggle.
                            st.set_is_album_blocked(!was_blocked);
                            let title = st.get_title().to_string();
                            let artist = st.get_artist().to_string();
                            let cover = st.get_artwork_url().to_string();
                            let res = if was_blocked {
                                crate::artist_blacklist::remove_album(&album_id)
                            } else {
                                crate::artist_blacklist::add_album(
                                    &album_id, &title, &artist, &cover, None,
                                )
                            };
                            match res {
                                Ok(()) => {
                                    seed_blacklist_status(&w);
                                    let msg = if was_blocked {
                                        qbz_i18n::t_args("Album \"{}\" unblocked", &[&title])
                                    } else {
                                        qbz_i18n::t_args("Album \"{}\" blocked", &[&title])
                                    };
                                    crate::toast::success_weak(&weak, msg);
                                }
                                Err(e) => {
                                    log::error!(
                                        "[qbz-slint] album block toggle failed: {e}"
                                    );
                                    st.set_is_album_blocked(was_blocked);
                                    let emsg = if was_blocked {
                                        qbz_i18n::t("Failed to unblock album")
                                    } else {
                                        qbz_i18n::t("Failed to block album")
                                    };
                                    crate::toast::error_weak(&weak, emsg);
                                }
                            }
                        }
                    }
                }
                // Artist card / grid overlay play button: Popular tracks, with
                // a studio-discography fallback when the artist has none (see
                // playback::play_artist).
                ("artist", "play") => playback::play_artist(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id.clone(),
                ),
                ("artist", "play-top") => playback::play_artist_top_tracks(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id.clone(),
                ),
                ("artist", "follow") => {
                    // Toggle the artist follow (= Qobuz artist favorite). State
                    // source = the in-memory artist fav cache (seeded by search +
                    // the artist page). Optimistic flip on the cache + every
                    // visible surface (search cards + the ArtistView heart),
                    // revert on network failure.
                    if let (Some(w), Ok(aid)) = (weak.upgrade(), id.parse::<u64>()) {
                        let following = crate::fav_cache::is_artist_favorite(aid);
                        let make = !following;
                        crate::fav_cache::set_artist(aid, make);
                        search::mark_artist_followed(&w, &id, make);
                        let ast = w.global::<ArtistState>();
                        if ast.get_id().as_str() == id.as_str() {
                            ast.set_is_following(make);
                        }
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let artist_id = id.clone();
                        handle.spawn(async move {
                            let res = if make {
                                runtime.core().add_favorite("artist", &artist_id).await
                            } else {
                                runtime.core().remove_favorite("artist", &artist_id).await
                            };
                            match res {
                                Ok(()) => {
                                    // reco: log the favorite only on ADD.
                                    if make {
                                        tokio::task::spawn_blocking(move || {
                                            crate::reco::log_favorite_artist(aid)
                                        });
                                    }
                                }
                                Err(e) => {
                                    log::error!(
                                        "[qbz-slint] toggle follow artist failed: {e}"
                                    );
                                    crate::fav_cache::set_artist(aid, following);
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        search::mark_artist_followed(&w, &artist_id, following);
                                        let ast = w.global::<ArtistState>();
                                        if ast.get_id().as_str() == artist_id.as_str() {
                                            ast.set_is_following(following);
                                        }
                                    });
                                }
                            }
                        });
                    }
                }
                // "Not interested" (reco-scoped dismissal — NOT the app-wide
                // blacklist): persist the dismissal, drop the card from the
                // Recommendations rails live, and backfill the freed slot from
                // the retained overflow. The artist stays visible everywhere
                // else (search/home/label pages); future paints exclude it via
                // the §B filter.
                ("artist", "not-interested") => {
                    if let Some(w) = weak.upgrade() {
                        let snapshot =
                            crate::external_reco::apply_artist_dismissal(&w, &image_cache, &id);
                        match snapshot {
                            Some((name, image)) => {
                                if let Ok(aid) = id.parse::<u64>() {
                                    crate::reco_dismiss::dismiss(aid, &name, &image);
                                }
                                crate::toast::info_weak(
                                    &weak,
                                    qbz_i18n::t_args(
                                        "{} won't appear in Recommendations anymore",
                                        &[&name],
                                    ),
                                );
                            }
                            None => {
                                // Dismissed from a non-reco surface (search /
                                // home / pinned card): nothing to remove live
                                // — resolve the display name, then persist.
                                let runtime = runtime.clone();
                                let weak = weak.clone();
                                let artist_id = id.clone();
                                handle.spawn(async move {
                                    let Ok(aid) = artist_id.parse::<u64>() else {
                                        return;
                                    };
                                    let (name, image) = runtime
                                        .core()
                                        .get_artist(aid)
                                        .await
                                        .map(|a| {
                                            (
                                                a.name,
                                                a.image
                                                    .and_then(|i| i.best().cloned())
                                                    .unwrap_or_default(),
                                            )
                                        })
                                        .unwrap_or_default();
                                    crate::reco_dismiss::dismiss(aid, &name, &image);
                                    let msg = if name.is_empty() {
                                        qbz_i18n::t("Artist dismissed from Recommendations")
                                    } else {
                                        qbz_i18n::t_args(
                                            "{} won't appear in Recommendations anymore",
                                            &[&name],
                                        )
                                    };
                                    let _ = weak.upgrade_in_event_loop(move |w| {
                                        crate::toast::info(&w, msg);
                                    });
                                });
                            }
                        }
                    }
                }
                // === Label landing actions ===============================
                ("label", "follow") => {
                    // Toggle the label favorite, optimistically flipping the
                    // header + any matching More-Labels card.
                    if let Some(w) = weak.upgrade() {
                        let make = !label::label_following_state(&w, &id);
                        label::mark_label_followed(&w, &id, make);
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let label_id = id.clone();
                        handle.spawn(async move {
                            let res = if make {
                                runtime.core().add_favorite("label", &label_id).await
                            } else {
                                runtime.core().remove_favorite("label", &label_id).await
                            };
                            if let Err(e) = res {
                                log::error!("[qbz-slint] toggle label favorite failed: {e}");
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    label::mark_label_followed(&w, &label_id, !make);
                                });
                            }
                        });
                    }
                }
                ("label", "play-top") => {
                    // Popular tracks are cached on the UI thread by
                    // apply_label_page; read them here (UI thread) + queue.
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::play_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            0,
                        );
                    }
                }
                // Label Popular Tracks multi-select: mode toggle + bulk bar.
                ("label", "select-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let on = w.global::<LabelState>().get_multi_select();
                        label::set_multi_select(&w, !on);
                    }
                }
                ("label", "select-all") => {
                    if let Some(w) = weak.upgrade() {
                        label::select_all(&w);
                    }
                }
                ("label", "clear") => {
                    if let Some(w) = weak.upgrade() {
                        label::clear_selection(&w);
                    }
                }
                ("label", "queue") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = label::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                }
                ("label", "play-next") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = label::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                }
                // Popular Tracks section menu + header overflow: ALL of the
                // label's popular tracks play-next / add-to-queue (the cached
                // list — same source as "play-top").
                ("label", "top-play-next") => {
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                }
                ("label", "top-queue") => {
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                }
                // Header shuffle: all popular tracks, xorshift-shuffled.
                ("label", "shuffle") => {
                    let tracks = label::top_tracks_for_play();
                    if tracks.is_empty() {
                        crate::toast::error_weak(&weak, "No popular tracks for this label");
                    } else {
                        playback::play_label_top_shuffled(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            id.clone(),
                        );
                    }
                }
                // Header overflow Share — Qobuz web-player label link (no
                // Song.link/Album.link equivalent exists for labels).
                ("label", "share") => {
                    let label_id = if id.is_empty() {
                        weak.upgrade()
                            .map(|w| w.global::<LabelState>().get_id().to_string())
                            .unwrap_or_default()
                    } else {
                        id.clone()
                    };
                    if !label_id.is_empty() {
                        share::copy_to_clipboard(share::qobuz_label_url(&label_id));
                        if let Some(w) = weak.upgrade() {
                            crate::toast::success(&w, qbz_i18n::t("Link copied"));
                        }
                    }
                }
                ("label", "add-to-playlist") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = label::selected_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                }
                ("label", "add-to-mixtape") => {
                    if let Some(w) = weak.upgrade() {
                        let items =
                            mixtape_items_from_qobuz_tracks(&label::selected_play_tracks(&w));
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            label::clear_selection(&w);
                        }
                    }
                }
                // More-Labels card click -> open that label's landing.
                ("label", "open") => {
                    if let Ok(label_id) = id.parse::<u64>() {
                        let name = weak
                            .upgrade()
                            .map(|w| label::more_label_name(&w, &id))
                            .unwrap_or_default();
                        nav::record(nav::NavEntry::Label {
                            id: label_id,
                            name: name.clone(),
                        });
                        navigate_label(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            label_id,
                            name,
                        );
                        if let Some(w) = weak.upgrade() {
                            update_nav_flags(&w);
                        }
                    }
                }
                // "See all" -> the full releases sub-view for the open label.
                ("label", "see-all-releases") => {
                    if let (Some(w), Ok(label_id)) = (weak.upgrade(), id.parse::<u64>()) {
                        let name = w.global::<LabelState>().get_name().to_string();
                        nav::record(nav::NavEntry::LabelReleases {
                            id: label_id,
                            name: name.clone(),
                        });
                        navigate_label_releases(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            label_id,
                            name,
                        );
                        update_nav_flags(&w);
                    }
                }
                ("track", "toggle-select") => {
                    // Plain / Ctrl+Click = single per-row toggle; Shift+Click =
                    // additive range from the per-surface anchor to the clicked
                    // row (1:1 with Tauri applyShiftRange — only ever adds). The
                    // anchor moves to the clicked row after either gesture. The
                    // surface id keys the anchor so a range never leaks across
                    // views; the model `match` mirrors the surface `match`.
                    if let Some(w) = weak.upgrade() {
                        let view = w.global::<NavState>().get_view();
                        let (model, surface) = match view {
                            ContentView::Album => {
                                (w.global::<AlbumState>().get_tracks(), selection::SURFACE_ALBUM)
                            }
                            ContentView::Playlist => (
                                w.global::<PlaylistState>().get_tracks(),
                                selection::SURFACE_PLAYLIST,
                            ),
                            ContentView::Label => (
                                w.global::<LabelState>().get_top_tracks(),
                                selection::SURFACE_LABEL,
                            ),
                            ContentView::Favorites => (
                                w.global::<FavoritesState>().get_tracks_visible(),
                                selection::SURFACE_FAVORITES,
                            ),
                            ContentView::Mix => (
                                w.global::<MixState>().get_tracks(),
                                selection::SURFACE_MIX,
                            ),
                            _ => (
                                w.global::<ArtistState>().get_top_tracks(),
                                selection::SURFACE_ARTIST,
                            ),
                        };
                        if let Some(vm) = model
                            .as_any()
                            .downcast_ref::<slint::VecModel<TrackItem>>()
                        {
                            let clicked = (0..vm.row_count()).find(|&i| {
                                vm.row_data(i)
                                    .map(|t| t.id.as_str() == id.as_str())
                                    .unwrap_or(false)
                            });
                            if let Some(clicked) = clicked {
                                let shift = keybindings::mods().2;
                                let anchor = if shift {
                                    selection::resolve_anchor(surface, vm, |t| t.id.to_string())
                                } else {
                                    None
                                };
                                match anchor {
                                    Some(anchor) => selection::apply_shift_range(
                                        vm,
                                        anchor,
                                        clicked,
                                        |t, v| t.selected = v,
                                    ),
                                    None => {
                                        if let Some(mut item) = vm.row_data(clicked) {
                                            item.selected = !item.selected;
                                            vm.set_row_data(clicked, item);
                                        }
                                    }
                                }
                                selection::set_anchor(surface, clicked, id.as_str());
                            }
                        }
                        match view {
                            ContentView::Album => album::recount_selected(&w),
                            ContentView::Artist => artist::recount_selected(&w),
                            ContentView::Playlist => playlist::recount_selected(&w),
                            ContentView::Favorites => favorites::recount_selected(&w),
                            ContentView::Mix => mix::recount_selected(&w),
                            ContentView::Label => label::recount_selected(&w),
                            _ => {}
                        }
                    }
                }
                // The mix tile sends id = mix kind, action = "open".
                ("mix", "open") => {
                    nav::record(nav::NavEntry::Mix { kind: id.clone() });
                    navigate_mix(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        id.clone(),
                    );
                    if let Some(w) = weak.upgrade() {
                        update_nav_flags(&w);
                    }
                }
                ("mix", "play-all") => {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = mix::current_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("mix", "shuffle") => {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = mix::shuffled_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("mix", "refresh") => {
                    // Re-load the current mix (re-fetch its tracks).
                    if let Some(w) = weak.upgrade() {
                        let kind = w.global::<MixState>().get_kind().to_string();
                        if !kind.is_empty() {
                            navigate_mix(
                                runtime.clone(),
                                weak.clone(),
                                &handle,
                                image_cache.clone(),
                                kind,
                            );
                        }
                    }
                }
                // Mix multi-select: mode toggle + bulk bar (select-all toggles
                // all/none; Ctrl+A select-all-only goes through the key handler).
                ("mix", "select-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let on = w.global::<MixState>().get_multi_select();
                        mix::set_multi_select(&w, !on);
                    }
                }
                ("mix", "select-all") => {
                    if let Some(w) = weak.upgrade() {
                        mix::select_all(&w);
                    }
                }
                ("mix", "clear") => {
                    if let Some(w) = weak.upgrade() {
                        mix::clear_selection(&w);
                    }
                }
                ("mix", "queue") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = mix::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                }
                ("mix", "play-next") => {
                    if let Some(w) = weak.upgrade() {
                        let tracks = mix::selected_play_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                }
                ("mix", "add-to-playlist") => {
                    if let Some(w) = weak.upgrade() {
                        let ids = mix::selected_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                }
                ("mix", "add-to-mixtape") => {
                    if let Some(w) = weak.upgrade() {
                        let items =
                            mixtape_items_from_qobuz_tracks(&mix::selected_play_tracks(&w));
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            mix::clear_selection(&w);
                        }
                    }
                }
                ("playlist", "cache") => {
                    if let Ok(pid) = id.parse::<u64>() {
                        offline_cache::cache_playlist(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            pid,
                        );
                    }
                }
                ("playlist", "play") => {
                    // Play a playlist by id NOW (replace the queue), from any
                    // playlist CARD overlay / context menu (Discover qobuzPlaylists,
                    // Search, Label) where no PlaylistView is open. The `play-all`
                    // arm below reads the open detail's PlaylistState, so it cannot
                    // serve a cold card play — this fetches the playlist by id.
                    playback::play_playlist(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id.clone(),
                    );
                }
                ("playlist", "play-all") => {
                    // LOCAL playlist detail — its own queue snapshot +
                    // offline-only stamp (D8); the offline sidecar view of
                    // a MIXED playlist (D11.a) AND the ONLINE mixed detail
                    // (Seam B: source-aware merged queue) share that
                    // snapshot; the pure-Qobuz path is unchanged below.
                    if let Some(w) = weak.upgrade() {
                        let ps = w.global::<PlaylistState>();
                        if ps.get_is_local()
                            || ps.get_offline_subset()
                            || playlist::is_mixed()
                        {
                            local_playlist::play_all(
                                &w,
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                false,
                            );
                            return;
                        }
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = playlist::current_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("playlist", "shuffle") => {
                    // Mixed pool shuffles as ONE list, local rows as
                    // equals (E9); the context stays the playlist id.
                    if let Some(w) = weak.upgrade() {
                        let ps = w.global::<PlaylistState>();
                        if ps.get_is_local()
                            || ps.get_offline_subset()
                            || playlist::is_mixed()
                        {
                            local_playlist::play_all(
                                &w,
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                true,
                            );
                            return;
                        }
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let tracks = playlist::shuffled_tracks();
                        playback::play_tracks(runtime, weak, handle, tracks, 0);
                    });
                }
                ("playlist", "queue") => {
                    if local_playlist::is_local_id(&id) {
                        local_playlist::enqueue_by_id(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                            false,
                        );
                        return;
                    }
                    playback::enqueue_playlist(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id,
                        false,
                    )
                }
                ("playlist", "play-next") => {
                    if local_playlist::is_local_id(&id) {
                        local_playlist::enqueue_by_id(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                            true,
                        );
                        return;
                    }
                    playback::enqueue_playlist(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id,
                        true,
                    )
                }
                ("playlist", "upload-to-qobuz") => {
                    // D8: convert a non-offline-only LOCAL playlist into a
                    // real Qobuz playlist (explicit user action, confirmed
                    // in the detail view — nothing ever auto-syncs).
                    if local_playlist::is_local_id(&id) {
                        local_playlist::upload_to_qobuz(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            image_cache.clone(),
                            id,
                        );
                    }
                }
                ("playlist", "favorite") => {
                    // Internal qbz library flag (Qobuz /favorite/create rejects
                    // playlist_ids). id-scoped: a CARD toggles ITS playlist, not
                    // the open one; the DB read picks the direction. `is_open`
                    // keeps the detail's optimistic heart in sync.
                    if let Some(w) = weak.upgrade() {
                        if local_playlist::is_local_id(&id) {
                            return;
                        }
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let is_open = w.global::<PlaylistState>().get_id().to_string() == id;
                        playlist_toggle_favorite_by_id(handle.clone(), weak.clone(), pid, is_open);
                    }
                }
                ("playlist", "copy") => {
                    // Copy a Qobuz playlist into the user's own playlists
                    // (create + add every track). id-scoped so a card copies ITS
                    // playlist; the detail passes its own id, so behavior is
                    // unchanged there (is_open flips PlaylistState.is-copied).
                    if let Some(w) = weak.upgrade() {
                        if local_playlist::is_local_id(&id) {
                            return;
                        }
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let is_open = w.global::<PlaylistState>().get_id().to_string() == id;
                        playlist_copy_by_id(runtime.clone(), weak.clone(), handle.clone(), pid, is_open);
                    }
                }
                ("playlist", "follow") => {
                    // Follow on Qobuz (subscribe). The DETAIL button emits
                    // "follow" as a toggle (id == open → flip current state); a
                    // CARD carries its follow-state and emits follow/unfollow
                    // explicitly, so a card "follow" always subscribes.
                    if let Some(w) = weak.upgrade() {
                        if local_playlist::is_local_id(&id) {
                            return;
                        }
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let is_open = w.global::<PlaylistState>().get_id().to_string() == id;
                        let follow = if is_open {
                            !w.global::<PlaylistState>().get_is_following()
                        } else {
                            true
                        };
                        playlist_set_follow_by_id(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            pid,
                            follow,
                            is_open,
                        );
                    }
                }
                ("playlist", "unfollow") => {
                    // Card-only: unfollow (unsubscribe) the given playlist.
                    if let Some(w) = weak.upgrade() {
                        if local_playlist::is_local_id(&id) {
                            return;
                        }
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let is_open = w.global::<PlaylistState>().get_id().to_string() == id;
                        playlist_set_follow_by_id(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            pid,
                            false,
                            is_open,
                        );
                    }
                }
                ("playlist", "select-toggle") => {
                    if let Some(w) = weak.upgrade() {
                        let on = w.global::<PlaylistState>().get_multi_select_mode();
                        playlist::set_multi_select(&w, !on);
                    }
                }
                ("playlist", "select-all") => {
                    if let Some(w) = weak.upgrade() {
                        playlist::select_all(&w);
                    }
                }
                ("playlist", "play-next-selected") | ("playlist", "queue-selected") => {
                    // Bulk Play next / Add to queue over the selection
                    // (Tauri's BulkActionBar split-button, spec §1.5) —
                    // source-aware: rows resolve through the merged queue
                    // snapshot (local/cached keep their source — the
                    // T2 fix-forward) or the pure-Qobuz Track cache.
                    if let Some(w) = weak.upgrade() {
                        let next = action == "play-next-selected";
                        let tracks = playlist::selected_queue_tracks(&w);
                        if tracks.is_empty() {
                            toast::error(&w, "Nothing playable in the selection");
                            return;
                        }
                        // Selection clears, mode stays on (LL precedent).
                        playlist::clear_selection(&w);
                        playback::enqueue_queue_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            next,
                        );
                    }
                }
                ("playlist", "add-selected-to-playlist") => {
                    // Bulk Add to playlist (spec §1.5). The picker is
                    // single-mode (catalog ids XOR local-mode refs), so:
                    // Qobuz rows ride the catalog flow; a selection with NO
                    // Qobuz rows rides the local-mode flow (library row ids
                    // — per-row parity for sidecar rows); a MIXED selection
                    // follows Tauri (Qobuz rows only, sidecar rows skipped +
                    // logged).
                    let Some(w) = weak.upgrade() else { return };
                    let rows = playlist::selected_rows(&w);
                    if rows.is_empty() {
                        return;
                    }
                    let mut qobuz_ids: Vec<String> = Vec::new();
                    let mut local_refs: Vec<String> = Vec::new();
                    for row in &rows {
                        match row.source.as_str() {
                            "local" => local_refs.push(row.id.clone()),
                            _ => {
                                if row.id.parse::<u64>().is_ok() {
                                    qobuz_ids.push(row.id.clone());
                                }
                            }
                        }
                    }
                    if !qobuz_ids.is_empty() {
                        if !local_refs.is_empty() {
                            log::info!(
                                "[qbz-slint] bulk add-to-playlist: mixed selection — {} sidecar row(s) skipped (single-mode picker; Tauri §1.5 behavior)",
                                local_refs.len()
                            );
                        }
                        playlist_picker::open_multi(&w, &qobuz_ids, false);
                    } else if !local_refs.is_empty() {
                        playlist_picker::open_multi(&w, &local_refs, true);
                    } else {
                        return;
                    }
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        let playlists = playlist_picker::load(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, playlists);
                        });
                    });
                }
                ("playlist", "remove-selected") => {
                    if let Some(w) = weak.upgrade() {
                        // LOCAL playlist — remove the selected rows from the
                        // library.db repo by stored position.
                        if w.global::<PlaylistState>().get_is_local() {
                            local_playlist::remove_selected(
                                &w,
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                image_cache.clone(),
                            );
                            return;
                        }
                        // QOBUZ detail (pure or mixed): split by row
                        // namespace — qobuz rows resolve to ptids, local
                        // rows to the local sidecar delete (Seam D).
                        let pid = w.global::<PlaylistState>().get_id().to_string();
                        let rows = playlist::selected_rows(&w);
                        if let (Ok(pid), false) = (pid.parse::<u64>(), rows.is_empty()) {
                            playlist_remove_rows(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                image_cache.clone(),
                                pid,
                                rows,
                            );
                        }
                    }
                }
                ("playlist", "set-artwork") => {
                    // Pick an image, copy it into the artwork cache, store
                    // it as the playlist's custom cover, then reload.
                    if let Some(w) = weak.upgrade() {
                        let pid = w.global::<PlaylistState>().get_id().to_string();
                        // LOCAL playlist — same flow, repo-backed.
                        if local_playlist::is_local_id(&pid) {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            let handle = handle.clone();
                            let image_cache = image_cache.clone();
                            handle.clone().spawn(async move {
                                let Some(file) = rfd::AsyncFileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                    .pick_file()
                                    .await
                                else {
                                    return;
                                };
                                let src = file.path().to_string_lossy().into_owned();
                                let lid = pid.clone();
                                let ok = tokio::task::spawn_blocking(move || {
                                    local_playlist::set_custom_artwork_blocking(&lid, &src)
                                        .is_some()
                                })
                                .await
                                .unwrap_or(false);
                                if ok {
                                    local_playlist::navigate(
                                        runtime, weak, &handle, image_cache, pid,
                                    );
                                }
                            });
                            return;
                        }
                        if let Ok(pid) = pid.parse::<u64>() {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            let handle = handle.clone();
                            let image_cache = image_cache.clone();
                            handle.clone().spawn(async move {
                                let Some(file) = rfd::AsyncFileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                    .pick_file()
                                    .await
                                else {
                                    return;
                                };
                                let src = file.path().to_string_lossy().into_owned();
                                let ok = tokio::task::spawn_blocking(move || {
                                    playlist::set_custom_artwork(pid, &src).is_some()
                                })
                                .await
                                .unwrap_or(false);
                                if ok {
                                    navigate_playlist(
                                        runtime, weak, &handle, image_cache, pid.to_string(),
                                    );
                                }
                            });
                        }
                    }
                }
                ("playlist", "clear-artwork") => {
                    if let Some(w) = weak.upgrade() {
                        let pid = w.global::<PlaylistState>().get_id().to_string();
                        // LOCAL playlist — clear the repo column + reload.
                        if local_playlist::is_local_id(&pid) {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            let handle = handle.clone();
                            let image_cache = image_cache.clone();
                            handle.clone().spawn(async move {
                                let lid = pid.clone();
                                tokio::task::spawn_blocking(move || {
                                    local_playlist::clear_custom_artwork_blocking(&lid);
                                })
                                .await
                                .ok();
                                local_playlist::navigate(
                                    runtime, weak, &handle, image_cache, pid,
                                );
                            });
                            return;
                        }
                        if let Ok(pid) = pid.parse::<u64>() {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            let handle = handle.clone();
                            let image_cache = image_cache.clone();
                            handle.clone().spawn(async move {
                                tokio::task::spawn_blocking(move || {
                                    playlist::clear_custom_artwork(pid);
                                })
                                .await
                                .ok();
                                navigate_playlist(
                                    runtime, weak, &handle, image_cache, pid.to_string(),
                                );
                            });
                        }
                    }
                }
                ("playlist", "edit") => {
                    // Open the edit modal, prefilled from the open playlist.
                    if let Some(w) = weak.upgrade() {
                        let ps = w.global::<PlaylistState>();
                        let pid = ps.get_id();
                        let name = ps.get_name();
                        let desc = ps.get_description();
                        let is_local = ps.get_is_local();
                        let offline_only = ps.get_offline_only();
                        let es = w.global::<EditPlaylistState>();
                        es.set_id(pid);
                        es.set_name(name);
                        es.set_description(desc);
                        es.set_is_local(is_local);
                        es.set_offline_only(offline_only);
                        es.set_open(true);
                    }
                }
                ("track", "move-up") | ("track", "move-down") => {
                    // Custom-order reorder (playlist view). Optimistic UI
                    // move, then persist the full order off-thread.
                    if let Some(w) = weak.upgrade() {
                        let up = action == "move-up";
                        let pid = w.global::<PlaylistState>().get_id().to_string();
                        // LOCAL playlist (B2): the move writes the repo's
                        // position order directly (no custom-order sidecar).
                        if local_playlist::is_local_id(&pid) {
                            local_playlist::move_row(&w, &handle, id.as_str(), up);
                        } else {
                            let orders = playlist::move_track(&w, id.as_str(), up);
                            if !orders.is_empty() {
                                if let Ok(pid) = pid.parse::<u64>() {
                                    handle.spawn(async move {
                                        tokio::task::spawn_blocking(move || {
                                            playlist::persist_custom(pid, orders);
                                        })
                                        .await
                                        .ok();
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        });
    }

    // Transport — wired through the NowPlayingState global callbacks.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_toggle_play(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::toggle_play_pause(runtime, weak, handle);
                });
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<NowPlayingState>().on_next(move || {
            let runtime = runtime.clone();
            let weak = weak.clone();
            let handle = handle.clone();
            handle.clone().spawn(async move {
                // NOTE: no cast-specific branch here. While casting, the local
                // next() flow runs — it moves the core cursor, refreshes the
                // now-playing card + queue, and calls play_audible, which casts
                // the new current track (the play_audible cast gate). Routing
                // next() through a cast-only path would advance the renderer but
                // leave the UI cursor stale (and then queue-click resolves
                // against the wrong index).
                playback::next(runtime, weak, handle);
            });
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<NowPlayingState>().on_previous(move || {
            let runtime = runtime.clone();
            let weak = weak.clone();
            let handle = handle.clone();
            handle.clone().spawn(async move {
                // See on_next: no cast branch — the local previous() flow keeps
                // the cursor + UI in sync and play_audible casts the new track.
                playback::previous(runtime, weak, handle);
            });
        });
    }
    {
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_seek(move |fraction| {
                let runtime = runtime.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::seek(runtime, handle, fraction);
                });
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_set_volume(move |fraction| {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::set_volume(runtime, weak, handle, fraction);
                });
            });
    }
    // Persist sidebar state / section-nav placement / volume (drag-end only)
    // to ui_prefs. These callbacks just touch the prefs file — no runtime.
    {
        let shell = window.global::<ShellState>();
        shell.on_persist_sidebar_state(|state| {
            let mut prefs = crate::ui_prefs::load();
            prefs.sidebar_state = state;
            crate::ui_prefs::save(&prefs);
        });
        shell.on_persist_nav(|enabled| {
            let mut prefs = crate::ui_prefs::load();
            prefs.nav_in_sidebar = enabled;
            crate::ui_prefs::save(&prefs);
        });
        shell.on_persist_nav_compact(|enabled| {
            let mut prefs = crate::ui_prefs::load();
            prefs.nav_header_compact = enabled;
            crate::ui_prefs::save(&prefs);
        });
        window.global::<NowPlayingState>().on_persist_volume(|fraction| {
            let mut prefs = crate::ui_prefs::load();
            prefs.volume = fraction.clamp(0.0, 1.0);
            crate::ui_prefs::save(&prefs);
        });
        // Remember the last SAFE top-level view for "where you left off".
        let weak = window.as_weak();
        shell.on_persist_view(move || {
            let Some(w) = weak.upgrade() else { return };
            let mut prefs = crate::ui_prefs::load();
            let mut dirty = false;
            // Legacy top-level key (offline-restore fallback).
            if let Some(key) = safe_view_key(w.global::<NavState>().get_view()) {
                if prefs.last_view != key {
                    prefs.last_view = key.to_string();
                    dirty = true;
                }
            }
            // Full entry for exact restore. Skip transient/config destinations
            // (a relaunch into the live-search results page or Settings is not
            // "where you left off"); those keep the prior last_nav.
            if let Some(entry) = nav::current() {
                let persistable =
                    !matches!(entry, nav::NavEntry::Search(_) | nav::NavEntry::Settings);
                if persistable {
                    if let Ok(json) = serde_json::to_string(&entry) {
                        if prefs.last_nav.as_deref() != Some(json.as_str()) {
                            prefs.last_nav = Some(json);
                            dirty = true;
                        }
                    }
                }
            }
            if dirty {
                crate::ui_prefs::save(&prefs);
            }
        });
    }

    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_toggle_mute(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::toggle_mute(runtime, weak, handle);
                });
            });
    }

    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_toggle_shuffle(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::toggle_shuffle(runtime, weak, handle);
                });
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<NowPlayingState>()
            .on_cycle_repeat(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    playback::cycle_repeat(runtime, weak, handle);
                });
            });
    }

    // Queue sidebar — build the controller and wire every callback.
    {
        let controller = queue::QueueController::new(
            app_runtime.clone(),
            window.as_weak(),
            tokio_rt.handle().clone(),
            settings_ctx.playback_prefs(),
        );
        // Publish it so the playback paths refresh the sidebar after every
        // queue mutation (play / skip / auto-advance / enqueue).
        playback::set_queue_controller(controller.clone());

        let qs = window.global::<QueueState>();
        {
            let c = controller.clone();
            qs.on_play_upcoming(move |index| c.play_upcoming(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_play_coverflow_upcoming(move |index| {
                c.play_coverflow_upcoming(index.max(0) as usize)
            });
        }
        {
            let c = controller.clone();
            qs.on_play_history(move |index| c.play_history(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_remove_upcoming(move |index| c.remove_upcoming(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_remove_all_after(move |index| c.remove_all_after(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_add_to_playlist(move |index| c.add_to_playlist(index.max(0) as usize));
        }
        {
            let c = controller.clone();
            qs.on_reorder(move |from, to| {
                c.reorder(from.max(0) as usize, to.max(0) as usize);
            });
        }
        {
            let c = controller.clone();
            qs.on_clear_queue(move || c.clear());
        }
        {
            let c = controller.clone();
            qs.on_toggle_now_playing_favorite(move || c.toggle_favorite());
        }
        {
            let c = controller.clone();
            qs.on_save_as_playlist(move || c.save_as_playlist());
        }
        {
            let c = controller.clone();
            qs.on_toggle_infinite_play(move || c.toggle_infinite_play());
        }
        {
            let c = controller.clone();
            qs.on_toggle_stop_after(move |id| c.toggle_stop_after(id.to_string()));
        }
        // Sleep timer (queue footer): a Rust-owned tokio task drives the countdown
        // and pauses playback at the deadline.
        {
            let runtime = app_runtime.clone();
            let weak = window.as_weak();
            let handle = tokio_rt.handle().clone();
            window
                .global::<SleepTimerActions>()
                .on_set(move |minutes| {
                    sleep_timer::set(runtime.clone(), weak.clone(), handle.clone(), minutes)
                });
        }
        {
            let weak = window.as_weak();
            window
                .global::<SleepTimerActions>()
                .on_cancel(move || sleep_timer::cancel(weak.clone()));
        }
        // Developer panel: in-app log viewer + the full diagnostics panel.
        log_viewer::install(&window, app_runtime.clone(), tokio_rt.handle().clone());
        diagnostics::install(&window, app_runtime.clone(), tokio_rt.handle().clone());
        // Report-an-issue: "Create issue report" opens the GitHub new-issue page.
        window.global::<ReportIssueActions>().on_create_issue(|| {
            let url = "https://github.com/vicrodh/qbz/issues/new?template=bug_report.yml";
            if let Err(e) = open::that(url) {
                log::warn!("[qbz-slint] open GitHub issues failed: {e}");
            }
        });
        // About QBZ (static seed + open-url) and What's New (fetch on open).
        about::install(&window, tokio_rt.handle().clone());
        whats_new::install(&window, tokio_rt.handle().clone());
        {
            let c = controller.clone();
            let weak = window.as_weak();
            qs.on_search_changed(move || {
                let query = weak
                    .upgrade()
                    .map(|w| w.global::<QueueState>().get_search_query().to_string())
                    .unwrap_or_default();
                c.search_changed(query);
            });
        }
        {
            let c = controller.clone();
            qs.on_prev_page(move || c.prev_page());
        }
        {
            let c = controller.clone();
            qs.on_next_page(move || c.next_page());
        }
        {
            let c = controller.clone();
            qs.on_set_tab(move |tab| c.set_tab(tab));
        }
        {
            let c = controller.clone();
            // On open, also re-pull favorites so the heart is accurate.
            qs.on_panel_opened(move || c.refresh_with_favorites());
        }
    }

    // Album track search — client-side filter, no backend round-trip.
    {
        let weak = window.as_weak();
        window
            .global::<AlbumActions>()
            .on_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    album::filter_tracks(&w, query.as_str());
                }
            });
    }

    // Album multi-select: the toolbar toggle next to the search box.
    {
        let weak = window.as_weak();
        window
            .global::<AlbumActions>()
            .on_toggle_multi_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<AlbumState>().get_multi_select();
                    album::set_multi_select(&w, !on);
                }
            });
    }

    // Album multi-select bulk bar — actions over the selected catalog rows.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<AlbumActions>()
            .on_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => album::select_all(&w),
                    "clear" => album::clear_selection(&w),
                    "queue" => {
                        let tracks = album::selected_play_tracks(&w);
                        if !tracks.is_empty() {
                            playback::enqueue_tracks(
                                runtime.clone(),
                                handle.clone(),
                                tracks,
                                false,
                            );
                        }
                    }
                    "play-next" => {
                        let tracks = album::selected_play_tracks(&w);
                        if !tracks.is_empty() {
                            playback::enqueue_tracks(
                                runtime.clone(),
                                handle.clone(),
                                tracks,
                                true,
                            );
                        }
                    }
                    "make-offline" => {
                        let tracks = album::selected_play_tracks(&w);
                        if !tracks.is_empty() {
                            offline_cache::cache_tracks(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                tracks,
                            );
                            album::clear_selection(&w);
                        }
                    }
                    "add-to-playlist" => {
                        let ids = album::selected_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    "add-to-favorites" => {
                        let ids = album::selected_ids(&w);
                        if ids.is_empty() {
                            return;
                        }
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            for id in &ids {
                                match runtime.core().add_favorite("track", id).await {
                                    Ok(()) => {
                                        if let Ok(tid) = id.parse::<u64>() {
                                            crate::fav_cache::set(tid, true);
                                        }
                                    }
                                    Err(e) => log::error!(
                                        "[qbz-slint] bulk favorite track {id} failed: {e}"
                                    ),
                                }
                            }
                            let _ = weak.upgrade_in_event_loop(|w| {
                                album::clear_selection(&w);
                                crate::toast::success(&w, "Added to favorites");
                            });
                        });
                    }
                    _ => {}
                }
            });
    }

    // Per-disc "Disc N" header ⋯ menu (Qobuz album) — each action is scoped to
    // that disc's tracks only, resolved from the album's stashed raw catalog
    // tracks. Reuses the SAME queue ops as the album-header buttons (play_tracks
    // / play_album_shuffled's xorshift / enqueue_tracks), just over the disc
    // subset rather than the whole album.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<AlbumActions>()
            .on_disc_action(move |disc, action| {
                let mut tracks = album::disc_play_tracks(disc);
                if tracks.is_empty() {
                    return;
                }
                match action.as_str() {
                    "play" => {
                        playback::play_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            0,
                        );
                    }
                    "shuffle" => {
                        // Same SystemTime-seeded xorshift Fisher-Yates as the
                        // album-header Shuffle (playback::play_album_shuffled),
                        // applied to the disc subset before play_tracks.
                        let mut seed = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_nanos() as u64)
                            .unwrap_or(1)
                            | 1;
                        for i in (1..tracks.len()).rev() {
                            seed ^= seed << 13;
                            seed ^= seed >> 7;
                            seed ^= seed << 17;
                            let j = (seed % (i as u64 + 1)) as usize;
                            tracks.swap(i, j);
                        }
                        playback::play_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                            0,
                        );
                    }
                    "queue" => {
                        playback::enqueue_tracks(
                            runtime.clone(),
                            handle.clone(),
                            tracks,
                            false,
                        );
                    }
                    "play-next" => {
                        playback::enqueue_tracks(
                            runtime.clone(),
                            handle.clone(),
                            tracks,
                            true,
                        );
                    }
                    other => {
                        log::warn!("[qbz-slint] album disc-action: unknown action {other}");
                    }
                }
            });
    }

    // Album external-database links (Last.fm / Discogs / MusicBrainz) — open
    // the prebuilt url (AlbumState.*-url) in the system browser. Mirrors the
    // ArtworkActions open-in-browser handler.
    window
        .global::<AlbumActions>()
        .on_open_external_link(|url| {
            if url.is_empty() {
                return;
            }
            if let Err(e) = open::that(url.as_str()) {
                log::error!("[qbz-slint] album external link open failed: {e}");
            }
        });

    // Booklet reader removed — the album booklet button now downloads the PDF
    // (booklet::download_booklet via the ("album","booklet") media action). The
    // BookletActions/BookletState globals + AlbumBookletModal.slint are unused
    // now (left in place; remove in a UI cleanup pass that recompiles qbz-ui).

    // Artist in-page search — client-side filter over Popular Tracks
    // and every release-section album.
    {
        let weak = window.as_weak();
        window
            .global::<ArtistActions>()
            .on_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    artist::filter_artist(&w, query.as_str());
                }
            });
    }

    // Artist per-section sort (persisted by release_type).
    {
        let weak = window.as_weak();
        window
            .global::<ArtistActions>()
            .on_set_section_sort(move |rt, sort| {
                if let Some(w) = weak.upgrade() {
                    artist::resort_section(&w, rt.as_str(), sort.as_str());
                }
            });
    }

    // Artist per-section load-more (capped to 4 pages; reuses get_releases_grid).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ArtistActions>()
            .on_load_more_section(move |rt| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let release_type = rt.to_string();
                if !artist::section_can_load_more(&release_type) {
                    return;
                }
                let artist_id = w.global::<ArtistState>().get_id().to_string();
                if artist_id.is_empty() {
                    return;
                }
                let offset = artist::section_loaded_count(&w, &release_type) as u32;
                let runtime = runtime.clone();
                let weak2 = weak.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    match artist::load_release_page(&runtime, &artist_id, &release_type, offset)
                        .await
                    {
                        Ok((cards, has_more)) => {
                            let image_cache = image_cache.clone();
                            let rt2 = release_type.clone();
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                let jobs =
                                    artist::append_release_page(&w, &rt2, cards, has_more);
                                artwork::spawn_loads(jobs, w.as_weak(), image_cache);
                            });
                        }
                        Err(e) => {
                            log::warn!("[qbz-slint] load-more {release_type} failed: {e}")
                        }
                    }
                });
            });
    }

    // Artist "See discography" — open the dedicated releases page pre-filtered.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ArtistActions>()
            .on_open_releases(move |rt| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let artist_id = w.global::<ArtistState>().get_id().to_string();
                let artist_name = w.global::<ArtistState>().get_name().to_string();
                if artist_id.is_empty() {
                    return;
                }
                let release_type = rt.to_string();
                nav::record(nav::NavEntry::ArtistReleases {
                    id: artist_id.clone(),
                    name: artist_name.clone(),
                    release_type: release_type.clone(),
                });
                navigate_artist_releases(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    artist_id,
                    artist_name,
                    release_type,
                );
                update_nav_flags(&w);
            });
    }

    // Dedicated discography page — infinite load-more.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ArtistReleasesActions>()
            .on_load_more(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let st = w.global::<ArtistReleasesState>();
                if st.get_load_more_loading() || !st.get_has_more() {
                    return;
                }
                let artist_id = st.get_id().to_string();
                let release_type = st.get_release_type().to_string();
                if artist_id.is_empty() {
                    return;
                }
                let offset = artist_releases::loaded_count(&w);
                st.set_load_more_loading(true);
                let runtime = runtime.clone();
                let weak2 = weak.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    match artist::load_release_page(&runtime, &artist_id, &release_type, offset)
                        .await
                    {
                        Ok((cards, has_more)) => {
                            let image_cache = image_cache.clone();
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                let jobs = artist_releases::apply_page(&w, cards, has_more, false);
                                artwork::spawn_loads(jobs, w.as_weak(), image_cache);
                            });
                        }
                        Err(e) => {
                            log::warn!("[qbz-slint] artist releases load-more failed: {e}");
                            let _ = weak2.upgrade_in_event_loop(|w| {
                                w.global::<ArtistReleasesState>().set_load_more_loading(false);
                            });
                        }
                    }
                });
            });
    }

    // Dedicated discography page — sort change (persisted, shared with index).
    {
        let weak = window.as_weak();
        window
            .global::<ArtistReleasesActions>()
            .on_set_sort(move |sort| {
                if let Some(w) = weak.upgrade() {
                    artist_releases::resort(&w, sort.as_str());
                }
            });
    }

    // Dedicated discography page — retry after a failed load.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ArtistReleasesActions>()
            .on_retry(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let st = w.global::<ArtistReleasesState>();
                let artist_id = st.get_id().to_string();
                let name = st.get_name().to_string();
                let release_type = st.get_release_type().to_string();
                if artist_id.is_empty() {
                    return;
                }
                navigate_artist_releases(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    artist_id,
                    name,
                    release_type,
                );
            });
    }

    // Artist Popular Tracks multi-select — the section toggle.
    {
        let weak = window.as_weak();
        window
            .global::<ArtistActions>()
            .on_toggle_top_tracks_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<ArtistState>().get_top_tracks_multi_select();
                    artist::set_multi_select(&w, !on);
                }
            });
    }

    // Artist Popular Tracks bulk bar — actions over the selected rows.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ArtistActions>()
            .on_top_tracks_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let artist_id = w.global::<ArtistState>().get_id().to_string();
                match action.as_str() {
                    "select-all" => artist::select_all(&w),
                    "clear" => artist::clear_selection(&w),
                    "play-next" => playback::enqueue_artist_top_selected(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                        artist::selected_ids(&w),
                        true,
                    ),
                    "queue" => playback::enqueue_artist_top_selected(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                        artist::selected_ids(&w),
                        false,
                    ),
                    "add-to-playlist" => {
                        let ids = artist::selected_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    "add-to-favorites" => {
                        let ids = artist::selected_ids(&w);
                        if ids.is_empty() {
                            return;
                        }
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            for id in &ids {
                                match runtime.core().add_favorite("track", id).await {
                                    Ok(()) => {
                                        if let Ok(tid) = id.parse::<u64>() {
                                            crate::fav_cache::set(tid, true);
                                        }
                                    }
                                    Err(e) => log::error!(
                                        "[qbz-slint] bulk favorite track {id} failed: {e}"
                                    ),
                                }
                            }
                            let _ = weak.upgrade_in_event_loop(|w| {
                                artist::clear_selection(&w);
                                crate::toast::success(&w, "Added to favorites");
                            });
                        });
                    }
                    "add-to-mixtape" => {
                        let items = mixtape_items_from_artist_selection(&w);
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            artist::clear_selection(&w);
                        }
                    }
                    _ => {}
                }
            });
    }

    // Artist Popular Tracks section "more" menu — all-tracks actions.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ArtistActions>()
            .on_top_tracks_menu_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let artist_id = w.global::<ArtistState>().get_id().to_string();
                if artist_id.is_empty() {
                    return;
                }
                match action.as_str() {
                    "next-all" => playback::enqueue_artist_top_selected(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                        artist::all_top_track_ids(&w),
                        true,
                    ),
                    "queue-all" => playback::enqueue_artist_top_selected(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                        artist::all_top_track_ids(&w),
                        false,
                    ),
                    "shuffle-all" => playback::play_artist_top_shuffled(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        artist_id,
                    ),
                    "playlist-all" => {
                        let ids = artist::all_top_track_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    _ => {}
                }
            });
    }

    // Artist network sidebar — no persistence. Default open, user can
    // close per-session, and reset_network_sidebar re-applies the open
    // state on every artist navigation (open unless the content area is
    // space-constrained — see reset_network_sidebar). The toggle
    // callback stays a no-op on the Rust side — Slint already flips
    // NetworkSidebarState.open directly in the click handler.
    window
        .global::<NetworkSidebarActions>()
        .on_toggle(|| {});

    // Network sidebar — typed click callbacks. Each delivers the
    // minimum payload the future target views (ArtistsByLocation,
    // LabelReleases, MusicianPage) will need. Logged-only until those
    // views land in Slint.
    // Location click — open ArtistsByLocationView using the cached
    // location params from the Origin metadata (area, genres, tags).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<NetworkSidebarActions>()
            .on_location_clicked(move |mbid| {
                let Some(params) = artist::location_params() else {
                    log::warn!(
                        "[qbz-slint] location clicked but no cached params (mbid={mbid})"
                    );
                    return;
                };
                nav::record(nav::NavEntry::Location {
                    mbid: params.mbid.clone(),
                    area_id: params.area_id.clone(),
                    area_name: params.area_name.clone(),
                    country: params.country.clone(),
                    genres: params.genres.clone(),
                    tags: params.tags.clone(),
                });
                navigate_location(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    params,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    // Label click — open LabelReleasesView.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<NetworkSidebarActions>()
            .on_label_clicked(move |id, name| {
                let Ok(label_id) = id.parse::<u64>() else {
                    log::warn!("[qbz-slint] label clicked: invalid id {id}");
                    return;
                };
                let name = name.to_string();
                nav::record(nav::NavEntry::Label {
                    id: label_id,
                    name: name.clone(),
                });
                navigate_label(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    label_id,
                    name,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    // artist-clicked actually navigates — the target view (artist page)
    // already exists in Slint, unlike LabelReleases / ArtistsByLocation /
    // MusicianPage. Same flow as the top-level on_open_artist handler.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<NetworkSidebarActions>()
            .on_artist_clicked(move |id| {
                let artist_id = id.to_string();
                nav::record(nav::NavEntry::Artist(artist_id.clone()));
                navigate_artist(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    artist_id,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    // Musician click — resolve the (name, role) first; if Qobuz has
    // a confirmed exact match, jump straight to that artist's page.
    // Otherwise open MusicianPageView (Contextual / Weak / None).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<NetworkSidebarActions>()
            .on_musician_clicked(move |name, role| {
                let name = name.to_string();
                let role = role.to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let image_cache = image_cache.clone();
                tokio::spawn(async move {
                    let resolved =
                        runtime.core().musicbrainz_resolve_musician(&name, &role).await;
                    match resolved {
                        Ok(r) if matches!(
                            r.confidence,
                            qbz_integrations::musicbrainz::MusicianConfidence::Confirmed
                        ) =>
                        {
                            if let Some(id) = r.qobuz_artist_id {
                                let artist_id = id.to_string();
                                let weak2 = weak.clone();
                                let _ = weak.clone().upgrade_in_event_loop(move |_| {
                                    nav::record(nav::NavEntry::Artist(artist_id.clone()));
                                });
                                navigate_artist(
                                    runtime,
                                    weak2,
                                    &handle,
                                    image_cache,
                                    id.to_string(),
                                );
                                return;
                            }
                            log::warn!(
                                "[qbz-slint] musician confirmed but no qobuz id"
                            );
                        }
                        Ok(_) => {
                            // Fall through to MusicianPageView for
                            // Contextual / Weak / None.
                        }
                        Err(e) => {
                            log::warn!("[qbz-slint] musician resolve failed: {e}");
                        }
                    }
                    nav::record(nav::NavEntry::Musician {
                        name: name.clone(),
                        role: role.clone(),
                    });
                    navigate_musician(runtime, weak, &handle, image_cache, name, role);
                });
            });
    }
    // discovery-dismissed — persist the rejection under the current
    // tag, then remove the row from the visible list.
    {
        let weak = window.as_weak();
        window
            .global::<NetworkSidebarActions>()
            .on_discovery_dismissed(move |mbid, name| {
                if let Some(w) = weak.upgrade() {
                    let tag = w
                        .global::<NetworkSidebarState>()
                        .get_discovery_tag()
                        .to_string()
                        .to_lowercase();
                    if !tag.is_empty() {
                        let normalized =
                            qbz_core::normalize_artist_name(name.as_str());
                        discovery_dismiss::dismiss(&tag, &normalized);
                    }
                    artist::remove_discovery_artist(&w, mbid.as_str());
                }
            });
    }

    // Track Info + Album Info modal actions (close / tab / navigation / play).
    // Navigation reuses the same handlers the rest of the app uses (open-artist
    // callback, network-sidebar musician resolve, navigate_label).
    {
        let runtime = app_runtime.clone();
        // -- Track Info --
        let weak = window.as_weak();
        window
            .global::<TrackInfoActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<TrackInfoState>().set_open(false);
                }
            });
        let weak = window.as_weak();
        window
            .global::<TrackInfoActions>()
            .on_open_artist(move |artist_id| {
                if let Some(w) = weak.upgrade() {
                    w.global::<TrackInfoState>().set_open(false);
                    w.invoke_open_artist(artist_id);
                }
            });
        let weak = window.as_weak();
        let runtime_l = runtime.clone();
        let handle_l = tokio_rt.handle().clone();
        let image_cache_l = image_cache.clone();
        window
            .global::<TrackInfoActions>()
            .on_open_label(move |label_id| {
                if let Some(w) = weak.upgrade() {
                    let name = w.global::<TrackInfoState>().get_label().to_string();
                    w.global::<TrackInfoState>().set_open(false);
                    if let Ok(id) = label_id.parse::<u64>() {
                        navigate_label(
                            runtime_l.clone(),
                            w.as_weak(),
                            &handle_l,
                            image_cache_l.clone(),
                            id,
                            name,
                        );
                    }
                }
            });
        let weak = window.as_weak();
        window
            .global::<TrackInfoActions>()
            .on_open_musician(move |name, role| {
                if let Some(w) = weak.upgrade() {
                    w.global::<TrackInfoState>().set_open(false);
                    w.global::<NetworkSidebarActions>()
                        .invoke_musician_clicked(name, role);
                }
            });
        // Immersive split Track Info panel: populate TrackInfoState for the
        // given track WITHOUT opening the floating modal (open stays false).
        let weak = window.as_weak();
        let runtime_l = runtime.clone();
        let handle_l = tokio_rt.handle().clone();
        window
            .global::<TrackInfoActions>()
            .on_load_inline(move |track_id| {
                if let Ok(id) = track_id.parse::<u64>() {
                    info_modals::load_track_info_inline(
                        runtime_l.clone(),
                        weak.clone(),
                        handle_l.clone(),
                        id,
                    );
                }
            });

        // -- Album Info --
        let weak = window.as_weak();
        window
            .global::<AlbumInfoActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<AlbumInfoState>().set_open(false);
                }
            });
        let weak = window.as_weak();
        window
            .global::<AlbumInfoActions>()
            .on_set_tab(move |tab| {
                if let Some(w) = weak.upgrade() {
                    w.global::<AlbumInfoState>().set_active_tab(tab);
                }
            });
        let weak = window.as_weak();
        let runtime_p = runtime.clone();
        let handle_p = tokio_rt.handle().clone();
        window
            .global::<AlbumInfoActions>()
            .on_play_track(move |id| {
                if let Some(w) = weak.upgrade() {
                    // Album view is the modal's context, so this plays the
                    // album starting at the chosen track (Tauri keeps the
                    // modal open on play).
                    playback::play_track_in_context(
                        &w,
                        runtime_p.clone(),
                        w.as_weak(),
                        handle_p.clone(),
                        &id,
                    );
                }
            });
        let weak = window.as_weak();
        let runtime_a = runtime.clone();
        let handle_a = tokio_rt.handle().clone();
        let image_cache_a = image_cache.clone();
        window
            .global::<AlbumInfoActions>()
            .on_open_label(move |label_id| {
                if let Some(w) = weak.upgrade() {
                    let name = w.global::<AlbumInfoState>().get_label().to_string();
                    w.global::<AlbumInfoState>().set_open(false);
                    if let Ok(id) = label_id.parse::<u64>() {
                        navigate_label(
                            runtime_a.clone(),
                            w.as_weak(),
                            &handle_a,
                            image_cache_a.clone(),
                            id,
                            name,
                        );
                    }
                }
            });
        let weak = window.as_weak();
        window
            .global::<AlbumInfoActions>()
            .on_open_musician(move |name, role| {
                if let Some(w) = weak.upgrade() {
                    w.global::<AlbumInfoState>().set_open(false);
                    w.global::<NetworkSidebarActions>()
                        .invoke_musician_clicked(name, role);
                }
            });
    }

    // Musician appearances pagination — Load more in
    // MusicianPageView appends the next 20 albums onto the existing
    // grid.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<MusicianActions>()
            .on_load_more(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let state = w.global::<MusicianState>();
                let name = state.get_name().to_string();
                let role = state.get_role().to_string();
                let offset = state.get_appearances().row_count() as u32;
                if name.is_empty() {
                    return;
                }
                state.set_load_more_loading(true);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let image_cache = image_cache.clone();
                handle.clone().spawn(async move {
                    match musician::load_more_appearances(&runtime, &name, &role, offset).await {
                        Ok((data, total)) => {
                            let jobs: Vec<artwork::ArtworkJob> = data
                                .iter()
                                .enumerate()
                                .filter(|(_, a)| !a.artwork_url.is_empty())
                                .map(|(i, a)| artwork::ArtworkJob {
                                    url: a.artwork_url.clone(),
                                    target: artwork::ArtworkTarget::MusicianAppearance {
                                        index: offset as usize + i,
                                    },
                                })
                                .collect();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                musician::append_appearances(&w, data, total);
                            });
                            artwork::spawn_loads(jobs, weak, image_cache);
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] musician load-more failed: {e}");
                            let _ = weak.upgrade_in_event_loop(|w| {
                                w.global::<MusicianState>().set_load_more_loading(false);
                            });
                        }
                    }
                });
            });
    }

    // Label album pagination — Load more in LabelReleasesView
    // appends the next page onto the grid.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LabelActions>()
            .on_load_more(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let state = w.global::<LabelState>();
                let Ok(label_id) = state.get_id().to_string().parse::<u64>() else {
                    return;
                };
                let offset = state.get_albums().row_count() as u32;
                state.set_load_more_loading(true);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    match label::load_more_albums(&runtime, label_id, offset).await {
                        Ok((data, total, has_more)) => {
                            let jobs: Vec<artwork::ArtworkJob> = data
                                .iter()
                                .enumerate()
                                .filter(|(_, a)| !a.artwork_url.is_empty())
                                .map(|(i, a)| artwork::ArtworkJob {
                                    url: a.artwork_url.clone(),
                                    target: artwork::ArtworkTarget::LabelAlbum {
                                        index: offset as usize + i,
                                    },
                                })
                                .collect();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                label::append_albums(&w, data, total, has_more);
                            });
                            artwork::spawn_loads(jobs, weak, image_cache);
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] label load-more failed: {e}");
                            let _ = weak.upgrade_in_event_loop(|w| {
                                w.global::<LabelState>().set_load_more_loading(false);
                            });
                        }
                    }
                });
            });
    }

    // Label releases sub-view toolbar — sort / Hi-Res filter /
    // group-by-artist / search. The markup updates the bound LabelState
    // property first; each callback just re-derives the rendered list
    // (local filter over the loaded catalog).
    {
        let weak = window.as_weak();
        window.global::<LabelActions>().on_set_sort(move |_| {
            if let Some(w) = weak.upgrade() {
                label::derive_releases(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LabelActions>().on_set_hires(move |_| {
            if let Some(w) = weak.upgrade() {
                label::derive_releases(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LabelActions>().on_set_group(move |_| {
            if let Some(w) = weak.upgrade() {
                label::derive_releases(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LabelActions>().on_search(move |_| {
            if let Some(w) = weak.upgrade() {
                label::derive_releases(&w);
            }
        });
    }


    // Immersive Suggestions panel actions (Checkpoint D — split-panel == 2).
    {
        // load(track-id) — entry + now-playing-change refresh. Reads the
        // artist-id + title off NowPlayingState (the panel only has the track
        // id) and kicks the live artist load (mirror of navigate_award).
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SuggestionsActions>()
            .on_load(move |track_id| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let np = w.global::<NowPlayingState>();
                let artist_id = np.get_artist_id().to_string();
                let track_id = track_id.to_string();
                let track_name = np.get_title().to_string();
                // Dedup: skip a reload when the panel already shows this artist
                // for this seed track (the changed-watcher can refire on
                // unrelated NowPlayingState churn).
                let ss = w.global::<SuggestionsState>();
                if ss.get_artist_id().as_str() == artist_id
                    && ss.get_seed_track_id().as_str() == track_id
                    && !track_id.is_empty()
                {
                    return;
                }
                navigate_suggestions(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    artist_id,
                    track_id,
                    track_name,
                );
            });
    }
    {
        // play / queue / play-next a curated artist playlist by id — reuse the
        // existing playback seams (same paths the playlist cards use).
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SuggestionsActions>()
            .on_play_playlist(move |playlist_id| {
                let id = playlist_id.to_string();
                if id.is_empty() {
                    return;
                }
                playback::play_playlist(runtime.clone(), weak.clone(), handle.clone(), id);
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SuggestionsActions>()
            .on_queue_playlist(move |playlist_id| {
                let id = playlist_id.to_string();
                if id.is_empty() {
                    return;
                }
                playback::enqueue_playlist(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                    false,
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SuggestionsActions>()
            .on_play_next_playlist(move |playlist_id| {
                let id = playlist_id.to_string();
                if id.is_empty() {
                    return;
                }
                playback::enqueue_playlist(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    id,
                    true,
                );
            });
    }
    {
        // play-track — play a single recommended track by id NOW.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SuggestionsActions>()
            .on_play_track(move |track_id| {
                let Ok(tid) = track_id.parse::<u64>() else {
                    return;
                };
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle2 = handle.clone();
                handle.spawn(async move {
                    match runtime.core().get_track(tid).await {
                        Ok(track) => {
                            playback::play_tracks(runtime, weak, handle2, vec![track], 0);
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] suggestions play-track {tid} failed: {e}");
                        }
                    }
                });
            });
    }

    // --- Playlist "Suggested Songs" section (T8) ----------------------------
    // 1:1 port of the Svelte PlaylistSuggestions component. The pool +
    // pagination + dedupe live in crate::playlist_suggestions; the nav actions
    // route through the shared media-action arms the playlist track rows use.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_activate(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_suggestions::activate(&w, runtime.clone(), handle.clone());
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_refresh(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_suggestions::refresh(&w, runtime.clone(), handle.clone());
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_add_track(move |track_id| {
                if let Some(w) = weak.upgrade() {
                    playlist_suggestions::add_track(
                        &w,
                        runtime.clone(),
                        handle.clone(),
                        track_id.to_string(),
                    );
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_play_track(move |track_id| {
                playlist_suggestions::play_track(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    track_id.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_dismiss_track(move |track_id| {
                if let Some(w) = weak.upgrade() {
                    playlist_suggestions::dismiss_track(
                        &w,
                        runtime.clone(),
                        handle.clone(),
                        track_id.to_string(),
                    );
                }
            });
    }
    {
        // show-info / go-album / go-artist reuse the shared media-action arms:
        // ("track","track-info") opens the Track Info modal; ("album"/"artist",
        // "open") navigate — the same routing the playlist track rows use.
        let weak = window.as_weak();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_show_info(move |track_id| {
                if let Some(w) = weak.upgrade() {
                    if !track_id.is_empty() {
                        w.invoke_media_action("track".into(), track_id, "track-info".into());
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_go_album(move |album_id| {
                if let Some(w) = weak.upgrade() {
                    if !album_id.is_empty() {
                        w.invoke_media_action("album".into(), album_id, "open".into());
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistSuggestionsActions>()
            .on_go_artist(move |artist_id| {
                if let Some(w) = weak.upgrade() {
                    if !artist_id.is_empty() {
                        w.invoke_media_action("artist".into(), artist_id, "open".into());
                    }
                }
            });
    }

    // Artist Blacklist Manager actions (Task 11). Mutations are synchronous
    // (in-memory set + single SQLite ops via the artist_blacklist wrapper), so
    // no tokio handle is needed; each callback runs on the event-loop thread.
    {
        // open() — the forward-open seam (T10's Settings content-filtering row
        // calls this). Records the nav entry, swaps the view, then loads the
        // blacklist. Mirrors OfflineManagerActions.on_open.
        let weak = window.as_weak();
        window.global::<BlacklistActions>().on_open(move || {
            nav::record(nav::NavEntry::BlacklistManager);
            if let Some(w) = weak.upgrade() {
                w.global::<NavState>()
                    .set_view(ContentView::BlacklistManager);
                update_nav_flags(&w);
            }
            blacklist_manager::load(weak.clone());
        });
    }
    {
        // back() — declared per the spec; the actual back chrome is the shared
        // header NavButtons (which drives nav::go_back). Wired here for any
        // future in-view trigger; routes through the same go-back path.
        let weak = window.as_weak();
        let app_runtime_bl = app_runtime.clone();
        let bl_handle = tokio_rt.handle().clone();
        let image_cache_bl = image_cache.clone();
        window.global::<BlacklistActions>().on_back(move || {
            if let Some((entry, scroll)) = nav::go_back() {
                let weak2 = weak.clone();
                arm_scroll_restore(&weak2, &entry, scroll);
                apply_entry(
                    entry,
                    &app_runtime_bl,
                    &weak2,
                    &bl_handle,
                    &image_cache_bl,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            }
        });
    }
    {
        let weak = window.as_weak();
        let bl_runtime_a = app_runtime.clone();
        let bl_handle_a = tokio_rt.handle().clone();
        let bl_image_cache_a = image_cache.clone();
        window
            .global::<BlacklistActions>()
            .on_artist_select(move |id| {
                let artist_id = id.to_string();
                nav::record(nav::NavEntry::Artist(artist_id.clone()));
                navigate_artist(
                    bl_runtime_a.clone(),
                    weak.clone(),
                    &bl_handle_a,
                    bl_image_cache_a.clone(),
                    artist_id,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_toggle_enabled(move || {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::toggle_enabled(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window.global::<BlacklistActions>().on_remove(move |id| {
            if let Some(w) = weak.upgrade() {
                blacklist_manager::remove(&w, id);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<BlacklistActions>().on_clear_all(move || {
            if let Some(w) = weak.upgrade() {
                blacklist_manager::clear_all(&w);
            }
        });
    }
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_search_changed(move |q| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::search_changed(&w, q.to_string());
                }
            });
    }
    // --- Album blacklist callbacks ---
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_set_tab(move |tab| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::set_tab(&w, tab);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_block_album(move |id, title, artist, cover| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::block_album(
                        &w,
                        id.to_string(),
                        title.to_string(),
                        artist.to_string(),
                        cover.to_string(),
                    );
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_remove_album(move |id| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::remove_album(&w, id.to_string());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_clear_all_albums(move || {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::clear_all_albums(&w);
                }
            });
    }
    // --- Reco-dismissal callbacks (the Recommendations tab) ---
    {
        let weak = window.as_weak();
        window
            .global::<BlacklistActions>()
            .on_remove_dismissed(move |id| {
                if let Some(w) = weak.upgrade() {
                    blacklist_manager::remove_dismissed(&w, id);
                }
            });
    }
    {
        let weak = window.as_weak();
        let bl_runtime_b = app_runtime.clone();
        let bl_handle_b = tokio_rt.handle().clone();
        let bl_image_cache_b = image_cache.clone();
        window
            .global::<BlacklistActions>()
            .on_album_select(move |id| {
                let album_id = id.to_string();
                nav::record(nav::NavEntry::Album(album_id.clone()));
                navigate_album(
                    bl_runtime_b.clone(),
                    weak.clone(),
                    &bl_handle_b,
                    bl_image_cache_b.clone(),
                    album_id,
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }

    // Offline Cache Manager actions.
    {
        let runtime = app_runtime.clone();
        let handle = tokio_rt.handle().clone();
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window.global::<OfflineManagerActions>().on_open(move || {
            nav::record(nav::NavEntry::OfflineManager);
            if let Some(w) = weak.upgrade() {
                w.global::<NavState>().set_view(ContentView::OfflineManager);
                update_nav_flags(&w);
            }
            offline_manager::load(weak.clone(), handle.clone());
        });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window.global::<OfflineManagerActions>().on_refresh(move || {
            offline_manager::load(weak.clone(), handle.clone());
        });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_select_artist(move |name| {
                offline_manager::select_artist(weak.clone(), handle.clone(), name.to_string());
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_set_sort(move |i| {
                offline_manager::set_sort(weak.clone(), handle.clone(), i);
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_toggle_failed(move || {
                offline_manager::toggle_failed(weak.clone(), handle.clone());
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<OfflineManagerActions>()
            .on_toggle_select(move |id| {
                if let Some(w) = weak.upgrade() {
                    offline_manager::toggle_select(&w, &id);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<OfflineManagerActions>()
            .on_select_all(move || {
                if let Some(w) = weak.upgrade() {
                    offline_manager::set_all_selected(&w, true);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<OfflineManagerActions>()
            .on_clear_selection(move || {
                if let Some(w) = weak.upgrade() {
                    offline_manager::set_all_selected(&w, false);
                }
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_bulk_redownload(move || {
                if let Some(w) = weak.upgrade() {
                    for id in offline_manager::selected_track_ids(&w) {
                        offline_cache::redownload_track(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                        );
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_bulk_remove(move || {
                if let Some(w) = weak.upgrade() {
                    for id in offline_manager::selected_track_ids(&w) {
                        offline_cache::remove_cached(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            id,
                        );
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_remove_track(move |id| {
                if let Ok(tid) = id.parse::<u64>() {
                    offline_cache::remove_cached(runtime.clone(), weak.clone(), handle.clone(), tid);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_remove_album(move |aid| {
                offline_cache::remove_album(weak.clone(), handle.clone(), aid.to_string());
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_redownload_track(move |id| {
                if let Ok(tid) = id.parse::<u64>() {
                    offline_cache::redownload_track(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        tid,
                    );
                }
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_redownload_album(move |aid| {
                offline_cache::redownload_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    aid.to_string(),
                    false,
                );
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_redownload_failed(move |aid| {
                offline_cache::redownload_album(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    aid.to_string(),
                    true,
                );
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_set_limit(move |gb| {
                offline_manager::set_limit(weak.clone(), handle.clone(), gb);
            });
    }
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window.global::<OfflineManagerActions>().on_clear_all(move || {
            offline_cache::clear_all(weak.clone(), handle.clone());
        });
    }
    {
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_open_folder(move || {
                offline_cache::open_folder(handle.clone());
            });
    }
    {
        let weak = window.as_weak();
        let runtime = runtime.clone();
        let handle = handle.clone();
        window
            .global::<OfflineManagerActions>()
            .on_play_track(move |id| {
                if let Ok(tid) = id.parse::<u64>() {
                    playback::play_track_now(runtime.clone(), weak.clone(), handle.clone(), tid);
                }
            });
    }
    }

    // Scene (location) view actions — open-artist routes to the
    // artist page, load-more validates the next page of candidates.
    {
        let weak = window.as_weak();
        window
            .global::<LocationViewActions>()
            .on_open_artist(move |id| {
                if id.is_empty() {
                    return;
                }
                if let Some(w) = weak.upgrade() {
                    w.invoke_open_artist(id);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocationViewActions>()
            .on_load_more(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let Some(params) = artist::location_params() else {
                    return;
                };
                let offset = w.global::<LocationViewState>().get_artists().row_count();
                w.global::<LocationViewState>().set_load_more_loading(true);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    match location_view::load_scene(&runtime, &params, offset).await {
                        Ok(data) => {
                            let jobs: Vec<artwork::ArtworkJob> = data
                                .artists
                                .iter()
                                .enumerate()
                                .filter(|(_, a)| !a.image_url.is_empty())
                                .map(|(i, a)| artwork::ArtworkJob {
                                    url: a.image_url.clone(),
                                    target: artwork::ArtworkTarget::LocationArtist {
                                        index: offset + i,
                                    },
                                })
                                .collect();
                            let total = data.total;
                            let artists = data.artists.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                location_view::append_scene(&w, artists, total);
                            });
                            artwork::spawn_loads(jobs, weak, image_cache);
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] scene load-more failed: {e}");
                            let _ = weak.upgrade_in_event_loop(|w| {
                                w.global::<LocationViewState>().set_load_more_loading(false);
                            });
                        }
                    }
                });
            });
    }

    // Discover tab switch (the in-view Home / Editor's Picks / For
    // You pill). Swaps the cached section set + re-fires artwork; For
    // You lazy-loads its dedicated sections on first open.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_select_tab(move |tab| {
                if let Some(w) = weak.upgrade() {
                    nav::record(nav::NavEntry::Discover {
                        tab: tab.to_string(),
                    });
                    let jobs = home::select_tab(&w, tab.as_str());
                    artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                    update_nav_flags(&w);
                    if tab.as_str() == "forYou" {
                        ensure_for_you_loaded(&runtime, &weak, &handle, &image_cache);
                    }
                    if tab.as_str() == "recommendations" {
                        external_reco::ensure_loaded(&runtime, &weak, &handle, &image_cache);
                    }
                }
            });
    }

    // Home "Recently Played Albums" rail "View all" -> the full page listing
    // the local play-history albums (record history, navigate, refresh the
    // nav flags).
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_open_recent_albums(move || {
                nav::record(nav::NavEntry::RecentAlbums);
                navigate_recent_albums(weak.clone(), &handle, image_cache.clone());
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_open_most_played_albums(move || {
                nav::record(nav::NavEntry::MostPlayedAlbums);
                navigate_most_played_albums(weak.clone(), &handle, image_cache.clone());
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<MostPlayedAlbumsActions>()
            .on_filter(move |q| {
                filter_most_played(weak.clone(), image_cache.clone(), q.to_string());
            });
    }

    // Qobuz Playlists rail "View all" -> the full-page playlist browse
    // (server-side tag + genre filtering). A fresh open resets the
    // category tab to All (Tauri parity); genre-filter and history
    // re-navigations preserve it.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_open_playlist_browse(move || {
                nav::record(nav::NavEntry::PlaylistBrowse);
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                playlist_browse::navigate(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    current_genre_filter(),
                    true,
                );
            });
    }

    // Recently-played rails refresh. `home-mounted` fires on every HomeView
    // (re)mount: re-read the LOCAL store into the rails IF a play was recorded
    // while Home was off-screen (dirty flag — a no-op otherwise, so mounting
    // Home stays free). While Home IS showing, playback refreshes the rails
    // directly (note_recent_store_changed). No polling anywhere.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<HomeActions>().on_home_mounted(move || {
            if RECENT_RAILS_DIRTY.load(std::sync::atomic::Ordering::Relaxed) {
                refresh_recent_rails(weak.clone(), &handle, image_cache.clone());
            }
        });
    }
    // Manual refresh (the toolbar button next to the nav cluster): an
    // unconditional local re-read of the recently-played rails on demand.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<HomeActions>().on_refresh_recent(move || {
            refresh_recent_rails(weak.clone(), &handle, image_cache.clone());
        });
    }
    // Library Albums (#566) header sort: reorder the cached favorite-albums
    // base list (0 recent / 1 first / 2 random) and re-push it + its covers.
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<HomeActions>().on_set_library_albums_sort(move |mode| {
            apply_library_albums_sort(weak.clone(), mode, image_cache.clone());
        });
    }

    // Qobuz Playlists category filter (multi-select, client-side). Toggling /
    // clearing a tag re-filters the cached playlists row and re-fires the
    // artwork for the new (filtered) positions — no re-fetch.
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_toggle_playlist_tag(move |slug| {
                if let Some(w) = weak.upgrade() {
                    let jobs = home::toggle_playlist_tag(&w, slug.as_str());
                    artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<HomeActions>()
            .on_clear_playlist_tags(move || {
                if let Some(w) = weak.upgrade() {
                    let jobs = home::clear_playlist_tags(&w);
                    artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                }
            });
    }

    // Discover section configurator (Slice 5) — gear opens the modal; toggle /
    // move / reset mutate the per-user prefs, persist, and re-render the active
    // tab from the cache (no refetch). The mutation handlers re-fire artwork for
    // newly-shown Home/Editor album sections, mirroring on_select_tab.
    {
        let weak = window.as_weak();
        window
            .global::<DiscoverActions>()
            .on_open_configurator(move || {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_open_configurator(&w);
                }
            });
    }
    // Recommendations-tab cache controls (unique to this tab).
    {
        let weak = window.as_weak();
        window
            .global::<ExternalRecoActions>()
            .on_set_cache_ttl(move |index| {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::set_reco_cache_ttl_index(&w, index);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<ExternalRecoActions>()
            .on_refresh_now(move || {
                external_reco::force_reload(&runtime, &weak, &handle, &image_cache);
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<DiscoverActions>()
            .on_close_configurator(move || {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_close_configurator(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<DiscoverActions>()
            .on_toggle_section(move |tab, id| {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_toggle(&w, tab.as_str(), id.as_str(), &image_cache);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<DiscoverActions>()
            .on_move_section(move |tab, id, dir| {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_move(&w, tab.as_str(), id.as_str(), dir, &image_cache);
                }
            });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<DiscoverActions>()
            .on_reset_tab(move |tab| {
                if let Some(w) = weak.upgrade() {
                    discover_prefs::on_reset(&w, tab.as_str(), &image_cache);
                }
            });
    }

    // Case-insensitive substring test backing the searchable QbzSelect
    // (Slint 1.16 has no `contains` builtin). Pure + stateless, so a single
    // registration at setup serves every searchable list.
    window
        .global::<TextUtil>()
        .on_contains_ci(|haystack: slint::SharedString, needle: slint::SharedString| {
            haystack
                .to_lowercase()
                .contains(needle.to_lowercase().as_str())
        });

    // Genre filter — selection is per context ("discover" / "favorites").
    // Toggling / clearing re-fetches the discover index (discover context)
    // or re-derives the favorites tab (favorites context).
    {
        let weak = window.as_weak();
        window
            .global::<GenreFilterActions>()
            .on_set_context(move |ctx| {
                genre_filter::set_context(ctx.as_str());
                if let Some(w) = weak.upgrade() {
                    genre_filter::apply_state(&w);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<GenreFilterActions>()
            .on_toggle(move |id| {
                let was_selected = genre_filter::selected_ids()
                    .iter()
                    .any(|x| x.to_string() == id.as_str());
                if !genre_filter::toggle(id.as_str()) {
                    return;
                }
                let Some(w) = weak.upgrade() else {
                    return;
                };
                genre_filter::apply_state(&w);
                // Library "All": client-side genre filter over the mixed feed.
                if genre_filter::current_context() == "library-all" {
                    let runtime_f = runtime.clone();
                    let weak_f = weak.clone();
                    let image_cache_f = image_cache.clone();
                    let id_f = id.to_string();
                    handle.spawn(async move {
                        if !was_selected {
                            if let Ok(gid) = id_f.parse::<u64>() {
                                genre_filter::load_descendants(&runtime_f, gid).await;
                            }
                        }
                        let _ = weak_f.upgrade_in_event_loop(move |w| {
                            genre_filter::apply_state(&w);
                            library_all::derive(&w);
                            let jobs = library_all::artwork_jobs(&w);
                            artwork::spawn_search_loads(jobs, w.as_weak(), image_cache_f.clone());
                        });
                    });
                    return;
                }
                // Favorites: client-side genre filter — re-derive the active
                // favorites tab instead of re-fetching the discover index.
                if genre_filter::current_context() == "favorites" {
                    let runtime_f = runtime.clone();
                    let weak_f = weak.clone();
                    let id_f = id.to_string();
                    handle.spawn(async move {
                        if !was_selected {
                            if let Ok(gid) = id_f.parse::<u64>() {
                                genre_filter::load_descendants(&runtime_f, gid).await;
                            }
                        }
                        let _ = weak_f.upgrade_in_event_loop(|w| {
                            genre_filter::apply_state(&w);
                            if w.global::<FavoritesState>().get_active_tab().as_str() == "albums" {
                                favorites::derive_albums(&w);
                            } else {
                                favorites::derive_tracks(&w);
                            }
                        });
                    });
                    return;
                }
                // When a "View all" browse page is showing (albums OR the
                // Qobuz Playlists page), the genre change re-fetches THAT
                // page; otherwise it reloads the Discover home index.
                let browse_target = current_browse_target(&w);
                let playlist_browse_showing = current_playlist_browse_showing(&w);
                if browse_target.is_none() && !playlist_browse_showing {
                    w.global::<HomeState>().set_loading(true);
                }
                let active = w.global::<HomeState>().get_active_tab().to_string();
                let id = id.to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let image_cache = image_cache.clone();
                let handle2 = handle.clone();
                handle.spawn(async move {
                    // On a newly-selected genre, eager-load its descendants
                    // so selected_names covers the child genres (favorites)
                    // and the tree shows counts.
                    if !was_selected {
                        if let Ok(gid) = id.parse::<u64>() {
                            genre_filter::load_descendants(&runtime, gid).await;
                            let _ = weak.upgrade_in_event_loop(|w| {
                                genre_filter::apply_state(&w);
                            });
                        }
                    }
                    if let Some((endpoint, title)) = browse_target {
                        discover_browse::navigate(
                            runtime.clone(),
                            weak.clone(),
                            &handle2,
                            image_cache.clone(),
                            endpoint,
                            title,
                            current_genre_filter(),
                        );
                    } else if playlist_browse_showing {
                        // Re-navigation preserves the page's selected tag
                        // (reset_tag = false).
                        playlist_browse::navigate(
                            runtime.clone(),
                            weak.clone(),
                            &handle2,
                            image_cache.clone(),
                            current_genre_filter(),
                            false,
                        );
                    } else {
                        reload_home(&runtime, &weak, &image_cache, active).await;
                    }
                });
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<GenreFilterActions>()
            .on_toggle_expand(move |id| {
                let now_expanded = genre_filter::toggle_expand(id.as_str());
                let Some(w) = weak.upgrade() else {
                    return;
                };
                genre_filter::apply_state(&w);
                // Lazy-load the node's children the first time it expands.
                if now_expanded {
                    if let Ok(gid) = id.to_string().parse::<u64>() {
                        if !genre_filter::children_loaded(gid) {
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                genre_filter::load_children(&runtime, gid).await;
                                let _ = weak.upgrade_in_event_loop(|w| {
                                    genre_filter::apply_state(&w);
                                });
                            });
                        }
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<GenreFilterActions>()
            .on_search(move |query| {
                genre_filter::set_search(query.as_str());
                if let Some(w) = weak.upgrade() {
                    genre_filter::apply_state(&w);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<GenreFilterActions>()
            .on_clear(move || {
                genre_filter::clear();
                let Some(w) = weak.upgrade() else {
                    return;
                };
                genre_filter::apply_state(&w);
                if genre_filter::current_context() == "library-all" {
                    library_all::derive(&w);
                    let jobs = library_all::artwork_jobs(&w);
                    artwork::spawn_search_loads(jobs, w.as_weak(), image_cache.clone());
                    return;
                }
                if genre_filter::current_context() == "favorites" {
                    if w.global::<FavoritesState>().get_active_tab().as_str() == "albums" {
                        favorites::derive_albums(&w);
                    } else {
                        favorites::derive_tracks(&w);
                    }
                    return;
                }
                let browse_target = current_browse_target(&w);
                let playlist_browse_showing = current_playlist_browse_showing(&w);
                if browse_target.is_none() && !playlist_browse_showing {
                    w.global::<HomeState>().set_loading(true);
                }
                let active = w.global::<HomeState>().get_active_tab().to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let image_cache = image_cache.clone();
                let handle2 = handle.clone();
                handle.spawn(async move {
                    if let Some((endpoint, title)) = browse_target {
                        discover_browse::navigate(
                            runtime.clone(),
                            weak.clone(),
                            &handle2,
                            image_cache.clone(),
                            endpoint,
                            title,
                            current_genre_filter(),
                        );
                    } else if playlist_browse_showing {
                        // Re-navigation preserves the page's selected tag
                        // (reset_tag = false).
                        playlist_browse::navigate(
                            runtime.clone(),
                            weak.clone(),
                            &handle2,
                            image_cache.clone(),
                            current_genre_filter(),
                            false,
                        );
                    } else {
                        reload_home(&runtime, &weak, &image_cache, active).await;
                    }
                });
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<GenreFilterActions>()
            .on_set_remember(move |v| {
                genre_filter::set_remember(v);
                if let Some(w) = weak.upgrade() {
                    genre_filter::apply_state(&w);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<GenreFilterActions>()
            .on_set_advanced(move |v| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                w.global::<GenreFilterState>().set_advanced(v);
                // First time advanced view opens, eager-load every
                // parent's children so the tree shows child counts.
                if v {
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    handle.spawn(async move {
                        genre_filter::load_all_parent_children(&runtime).await;
                        let _ = weak.upgrade_in_event_loop(|w| {
                            genre_filter::apply_state(&w);
                        });
                    });
                }
            });
    }

    // Header nav-menu navigation — currently routes the Library
    // dropdown rows into Library > Favorites tabs.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_header_menu_navigate(move |route| {
            if route == "home" {
                if let Some(w) = weak.upgrade() {
                    w.global::<NavState>().set_view(ContentView::Home);
                }
                return;
            }
            // My QBZ — Mixtapes / Collections index grids (read-only slice).
            // Record history + navigate (loads via myqbz::navigate), mirroring
            // the Favorites / Local Library per-route pattern.
            if route == "myqbz-mixtapes" {
                nav::record(nav::NavEntry::Mixtapes);
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                myqbz::navigate(
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                    qbz_models::mixtape::CollectionKind::Mixtape,
                );
                return;
            }
            if route == "myqbz-collections" {
                nav::record(nav::NavEntry::Collections);
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                myqbz::navigate(
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                    qbz_models::mixtape::CollectionKind::Collection,
                );
                return;
            }
            // Discover tabs — switch to Home and select the tab. The
            // section sets are already cached from the initial load,
            // so this just swaps the visible set + re-fires artwork.
            if let Some(tab) = route.strip_prefix("discover-") {
                let tab = tab.to_string();
                if let Some(w) = weak.upgrade() {
                    nav::record(nav::NavEntry::Discover { tab: tab.clone() });
                    w.global::<NavState>().set_view(ContentView::Home);
                    let jobs = home::select_tab(&w, &tab);
                    artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
                    update_nav_flags(&w);
                    if tab == "forYou" {
                        ensure_for_you_loaded(&runtime, &weak, &handle, &image_cache);
                    }
                    if tab == "recommendations" {
                        external_reco::ensure_loaded(&runtime, &weak, &handle, &image_cache);
                    }
                }
                return;
            }
            if route.as_str() == "favorites-all" {
                nav::record(nav::NavEntry::Favorites {
                    tab: "all".to_string(),
                });
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                navigate_library_all(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                );
                return;
            }
            if let Some(tab) = favorites::FavTab::from_route(route.as_str()) {
                let tab_id = route.strip_prefix("favorites-").unwrap_or("tracks");
                nav::record(nav::NavEntry::Favorites {
                    tab: tab_id.to_string(),
                });
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                navigate_favorites(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    tab,
                    tab_id,
                );
                return;
            }
            // Local Library tabs — same per-tab history pattern as Favorites.
            if let Some(tab) = local_library::LibTab::from_route(route.as_str()) {
                nav::record(nav::NavEntry::LocalLibrary {
                    tab: tab.tab_id().to_string(),
                });
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                navigate_local_library(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    tab,
                );
            }
        });
    }

    // Local Library — in-view tab bar (select-tab) + the gear button
    // (open-settings -> Settings > Local Library). Same per-tab history
    // pattern as Favorites.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_select_tab(move |tab_id| {
                if let Some(tab) = local_library::LibTab::from_tab_id(tab_id.as_str()) {
                    nav::record(nav::NavEntry::LocalLibrary {
                        tab: tab.tab_id().to_string(),
                    });
                    if let Some(w) = weak.upgrade() {
                        update_nav_flags(&w);
                    }
                    navigate_local_library(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                        tab,
                    );
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_open_settings(move || {
                // Management/maintenance/danger live under Settings > Local
                // Library — pre-select that sub-section (index 4). The panel's
                // `init` lazy-loads the folder list.
                nav::record(nav::NavEntry::Settings);
                if let Some(w) = weak.upgrade() {
                    w.global::<SettingsState>().set_section(4);
                    w.global::<NavState>().set_view(ContentView::Settings);
                    update_nav_flags(&w);
                }
            });
    }
    // Settings > Local Library — folder management + maintenance + danger.
    // (Scan callbacks scan-all/scan-folder/stop-scan are wired with Slice B.)
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_load(move || local_library_settings::load_folders(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_add_folder(move || local_library_settings::add_folder(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_remove_folders(move || {
                local_library_settings::remove_folders(weak.clone(), handle.clone())
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_remove_folder(move |id| {
                local_library_settings::remove_folder(weak.clone(), handle.clone(), id as i64)
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LibraryManageActions>()
            .on_toggle_folder_select(move |id| {
                local_library_settings::toggle_select(weak.clone(), id as i64)
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_edit_folder(move |id| {
                local_library_settings::edit_folder(weak.clone(), handle.clone(), id as i64)
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_save_folder_settings(move |id, alias, enabled, is_network, fs_type, user_override| {
                local_library_settings::save_folder_settings(
                    weak.clone(),
                    handle.clone(),
                    id as i64,
                    alias.to_string(),
                    enabled,
                    is_network,
                    fs_type.to_string(),
                    user_override,
                )
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_change_folder_path(move |id| {
                local_library_settings::change_folder_path(weak.clone(), handle.clone(), id as i64)
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_cleanup_missing(move || {
                local_library_settings::cleanup_missing(weak.clone(), handle.clone())
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_clear_library(move || {
                local_library_settings::clear_library(weak.clone(), handle.clone())
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LibraryManageActions>()
            .on_set_filter(move |_q| local_library_settings::set_filter(weak.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_scan_all(move || local_library_settings::scan_all(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LibraryManageActions>()
            .on_scan_folder(move |id| {
                local_library_settings::scan_folder(weak.clone(), handle.clone(), id as i64)
            });
    }
    {
        window
            .global::<LibraryManageActions>()
            .on_stop_scan(move || local_library_settings::stop_scan());
    }

    // Settings > Integrations — scrobblers (Last.fm + ListenBrainz). The auth
    // flows + the now-playing/scrobble fire live in `scrobble`; the persisted
    // store is the per-user `scrobbler_settings.db`.
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_load(move || scrobble::load(weak.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_enable_toggle(move |b| scrobble::enable_toggle(weak.clone(), b));
    }
    {
        window
            .global::<ScrobbleActions>()
            .on_collapse_toggle(move |b| scrobble::collapse_toggle(b));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_enable_toggle(move |b| scrobble::lastfm_enable_toggle(weak.clone(), b));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_connect(move || scrobble::lastfm_connect(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_open_auth_url(move || scrobble::lastfm_open_auth_url(weak.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_confirm(move || scrobble::lastfm_confirm(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_lastfm_disconnect(move || scrobble::lastfm_disconnect(weak.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_listenbrainz_enable_toggle(move |b| {
                scrobble::listenbrainz_enable_toggle(weak.clone(), b)
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<ScrobbleActions>()
            .on_listenbrainz_set_token(move |tok| {
                scrobble::listenbrainz_set_token(weak.clone(), handle.clone(), tok.to_string())
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<ScrobbleActions>()
            .on_listenbrainz_disconnect(move || scrobble::listenbrainz_disconnect(weak.clone()));
    }

    // Tag editor (local album metadata) — open via on_media_action("album",
    // "edit"); these wire the modal's own actions.
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_close(move || tag_editor::close_tag_editor(weak.clone()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<TagEditorActions>()
            .on_save(move || tag_editor::save_tags(weak.clone(), handle.clone(), image_cache.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_set_persistence(move |i| {
                if let Some(w) = weak.upgrade() {
                    let s = w.global::<TagEditorState>();
                    // Ignore selecting Direct when unavailable (CUE album).
                    if i == 1 && !s.get_can_direct_write() {
                        s.set_persistence_index(0);
                    } else {
                        s.set_persistence_index(i);
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_set_provider(move |i| {
                if let Some(w) = weak.upgrade() {
                    w.global::<TagEditorState>().set_remote_provider_index(i);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<TagEditorActions>()
            .on_search_remote(move || tag_editor::search_remote(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_select_result(move |id| tag_editor::select_result(weak.clone(), id.to_string()));
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<TagEditorActions>()
            .on_apply_remote(move || tag_editor::apply_remote(weak.clone(), handle.clone()));
    }
    {
        let weak = window.as_weak();
        window
            .global::<TagEditorActions>()
            .on_open_in_browser(move || tag_editor::open_in_browser(weak.clone()));
    }

    // Dedicated Local album view actions (play / shuffle / edit / add / version).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_play_all(move || {
            if let Some(w) = weak.upgrade() {
                let tracks = local_library::current_album_version_tracks(&w);
                playback::play_local_tracks(runtime.clone(), weak.clone(), handle.clone(), tracks, 0, false);
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_shuffle(move || {
            if let Some(w) = weak.upgrade() {
                let tracks = local_library::current_album_version_tracks(&w);
                playback::play_local_tracks(runtime.clone(), weak.clone(), handle.clone(), tracks, 0, true);
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_play_track(move |id| {
            if let Some(w) = weak.upgrade() {
                let tracks = local_library::current_album_version_tracks(&w);
                let start = id
                    .parse::<i64>()
                    .ok()
                    .and_then(|tid| tracks.iter().position(|t| t.id == tid))
                    .unwrap_or(0);
                playback::play_local_tracks(runtime.clone(), weak.clone(), handle.clone(), tracks, start, false);
            }
        });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_edit_tags(move || {
            if let Some(w) = weak.upgrade() {
                let idx = w.global::<LocalAlbumState>().get_version_index();
                if let Some(dir) = local_library::album_version_dir(idx) {
                    tag_editor::open_tag_editor(weak.clone(), handle.clone(), dir.clone(), dir);
                }
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_add_to_playlist(move || {
            if let Some(w) = weak.upgrade() {
                let tracks = local_library::current_album_version_tracks(&w);
                let refs: Vec<String> = tracks.iter().map(local_picker_ref).collect();
                if !refs.is_empty() {
                    playlist_picker::open_multi(&w, &refs, true);
                    let runtime = runtime.clone();
                    let weak2 = weak.clone();
                    handle.spawn(async move {
                        let pls = playlist_picker::load(&runtime).await;
                        let _ = weak2.upgrade_in_event_loop(move |w| {
                            playlist_picker::apply(&w, pls);
                        });
                    });
                }
            }
        });
    }
    {
        // Per-row context-menu actions on the local album detail (play-next /
        // queue / add-to-playlist / add-to-mixtape / favorite) — resolved
        // against the open version's track cache; "play" stays on
        // LocalAlbumActions.play-track.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalAlbumActions>()
            .on_track_menu_action(move |id, action| {
                let Some(w) = weak.upgrade() else { return };
                let tracks = local_library::current_album_version_tracks(&w);
                let Some(row) = tracks.iter().find(|t| t.id.to_string() == id.as_str())
                else {
                    return;
                };
                match action.as_str() {
                    "play-next" | "queue" => {
                        playback::enqueue_local_tracks(
                            runtime.clone(),
                            handle.clone(),
                            vec![row.clone()],
                            action.as_str() == "play-next",
                        );
                    }
                    "add-to-playlist" => {
                        playlist_picker::open_multi(&w, &[local_picker_ref(row)], true);
                        let runtime = runtime.clone();
                        let weak2 = weak.clone();
                        handle.spawn(async move {
                            let pls = playlist_picker::load(&runtime).await;
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                playlist_picker::apply(&w, pls);
                            });
                        });
                    }
                    "add-to-mixtape" => {
                        // Single-row Add to Mixtape/Collection on the local
                        // album detail (spec §3.1) — the row is already
                        // resolved from the open version's track cache.
                        let items =
                            myqbz_add::track_items_from_local(std::slice::from_ref(row));
                        open_add_to_mixtape(weak.clone(), handle.clone(), items);
                    }
                    "favorite" => {
                        // qobuz_download rows only (the menu gates the entry);
                        // toggle by the REAL Qobuz id, never the local row id
                        // (spec §3.2 — Tauri's latent bug, not ported).
                        match row.qobuz_track_id {
                            Some(qid) => toggle_track_favorite(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                qid.to_string(),
                            ),
                            None => log::debug!(
                                "[qbz-slint] favorite: album row {id} has no qobuz_track_id"
                            ),
                        }
                    }
                    "go-to-album" | "go-to-artist" => {
                        // Owner improvement over Tauri — source-routed in
                        // local_row_goto. On this surface "Go to album"
                        // reopens the open album for local rows (Qobuz
                        // album-view parity, where the entry also exists);
                        // qobuz_download rows reach their REAL Qobuz pages.
                        local_row_goto(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            row.clone(),
                            action.as_str() == "go-to-artist",
                        );
                    }
                    _ => {
                        log::debug!(
                            "[qbz-slint] unhandled local album track action: {id} {action}"
                        );
                    }
                }
            });
    }
    {
        // Add the whole local album to a Mixtape/Collection. Builds the
        // `album` payload (source "local", no artwork_url — 1:1 PSD) from the
        // LocalAlbumState header + the current version's track count.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<LocalAlbumActions>().on_add_to_mixtape(move || {
            if let Some(w) = weak.upgrade() {
                let st = w.global::<LocalAlbumState>();
                let id = st.get_id().to_string();
                if id.is_empty() {
                    return;
                }
                let tracks = local_library::current_album_version_tracks(&w);
                let item = myqbz_add::AddItem {
                    item_type: "album".into(),
                    source: "local".into(),
                    source_item_id: id,
                    title: st.get_title().to_string(),
                    subtitle: {
                        let a = st.get_artist().to_string();
                        (!a.is_empty()).then_some(a)
                    },
                    artwork_url: None,
                    year: None,
                    track_count: (!tracks.is_empty()).then_some(tracks.len() as i32),
                };
                open_add_to_mixtape(weak.clone(), handle.clone(), vec![item]);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LocalAlbumActions>().on_select_version(move |i| {
            if let Some(w) = weak.upgrade() {
                local_library::apply_album_version(&w, i);
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LocalAlbumActions>().on_search(move |q| {
            local_library::search_album(weak.clone(), q.to_string());
        });
    }
    {
        // Per-disc "Disc N" header ⋯ menu (local album) — scoped to that disc's
        // tracks only, resolved from the open version's track cache. Reuses the
        // SAME local queue ops as the header play-all / shuffle buttons
        // (play_local_tracks, shuffle flag) and the per-row menu's
        // enqueue_local_tracks, just over the disc subset.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalAlbumActions>()
            .on_disc_action(move |disc, action| {
                let Some(w) = weak.upgrade() else { return };
                let tracks = local_library::current_album_disc_tracks(&w, disc);
                if tracks.is_empty() {
                    return;
                }
                match action.as_str() {
                    "play" => playback::play_local_tracks(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        tracks,
                        0,
                        false,
                    ),
                    "shuffle" => playback::play_local_tracks(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        tracks,
                        0,
                        true,
                    ),
                    "queue" => playback::enqueue_local_tracks(
                        runtime.clone(),
                        handle.clone(),
                        tracks,
                        false,
                    ),
                    "play-next" => playback::enqueue_local_tracks(
                        runtime.clone(),
                        handle.clone(),
                        tracks,
                        true,
                    ),
                    other => {
                        log::warn!("[qbz-slint] local disc-action: unknown action {other}");
                    }
                }
            });
    }

    // Local Library — Albums tab controls (search / sort re-query page 1;
    // load-more pages on scroll; retry) + the shared AlbumCollectionView's
    // open / per-card actions (album-detail + playback land with later slices).
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_search(move |_query| {
                // Two-way bound to albums-search; re-derive in memory (full-load).
                if let Some(w) = weak.upgrade() {
                    local_library::derive_albums(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_window_changed(move |first, last| {
                // Windowed albums grid: dispatch covers for the reported row
                // band and evict the ones far outside it.
                if let Some(w) = weak.upgrade() {
                    local_library::albums_window_changed(&w, first, last);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_set_sort(move |sort| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_albums_sort(sort);
                    local_library::derive_albums(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_set_group(move |mode| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_albums_group(mode);
                    local_library::derive_albums(&w);
                }
            });
    }
    {
        // Album-identity mode (folder|metadata): the group KEY changes, so a
        // client-side derive is not enough — persist, reload the Albums set,
        // and invalidate the Artists tab (its album cache groups the same
        // way). Header dropdown + Settings row both land here.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_albums_set_id_mode(move |mode| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_albums_id_mode(mode.into());
                    crate::locallibrary_prefs::save(&w);
                    local_library::invalidate_artists(&w);
                    local_library::reload_albums(w.as_weak(), handle.clone(), image_cache.clone());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_set_view(move |mode| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_albums_view_mode(mode);
                    // Switching to the (non-windowed) list view needs covers
                    // the grid's window may have evicted.
                    local_library::albums_view_mode_changed(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_filter_changed(move || {
                if let Some(w) = weak.upgrade() {
                    local_library::derive_albums(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_clear_filter(move || {
                if let Some(w) = weak.upgrade() {
                    local_library::clear_album_filter(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_albums_retry(move || {
                local_library::reload_albums(weak.clone(), handle.clone(), image_cache.clone());
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_open_album(move |id| {
                nav::record(nav::NavEntry::LocalAlbum(id.to_string()));
                navigate_local_album(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    id.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_open_artist(move |name| {
                // `name` is the artist NAME (local artists have no id).
                open_local_artist(&runtime, &weak, &handle, &image_cache, name.to_string());
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_album_action(move |id, action| match action.as_str() {
                "play" => {
                    // The whole album becomes the queue and auto-advances.
                    playback::play_local_album(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        id.to_string(),
                        None,
                    );
                }
                "toggle-select" => {
                    if let Some(w) = weak.upgrade() {
                        local_library::toggle_album_select(&w, id.as_str());
                    }
                }
                "favorite" => {
                    if let Some(w) = weak.upgrade() {
                        local_library::toggle_album_favorite(&w, id.as_str());
                    }
                }
                "play-next" | "queue" => {
                    // Single-album play-next / queue (#636 — this arm used to
                    // be a "queue slice pending" stub): resolve the album's
                    // tracks source-aware (local folders, the same
                    // resolver `play` uses) and enqueue the whole album
                    // without starting playback.
                    let play_next = action.as_str() == "play-next";
                    let runtime = runtime.clone();
                    let handle2 = handle.clone();
                    let album_id = id.to_string();
                    handle.spawn(async move {
                        let rows = tokio::task::spawn_blocking(move || {
                            local_library::fetch_album_tracks_blocking(&album_id)
                        })
                        .await
                        .unwrap_or_default();
                        playback::enqueue_local_tracks(runtime, handle2, rows, play_next);
                    });
                }
                _ => {
                    log::debug!("[qbz-slint] unhandled local album action: {id} {action}");
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_search(move |_query| {
                // The query is two-way bound to tracks-search; reload page 1.
                local_library::reload_tracks(weak.clone(), handle.clone());
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_load_more(move || {
                local_library::load_more_tracks(weak.clone(), handle.clone());
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_retry(move || {
                local_library::reload_tracks(weak.clone(), handle.clone());
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_track_action(move |id, action| {
                match action.as_str() {
                    "play" => {
                        if let Ok(row_id) = id.parse::<i64>() {
                            // Queue the already-loaded rows (instant — no DB
                            // re-query / cover-fill that delayed the queue) so
                            // playback continues down the list from the click.
                            let tracks = local_library::tracks_current_snapshot();
                            if !tracks.is_empty() {
                                let start = tracks
                                    .iter()
                                    .position(|t| t.id == row_id)
                                    .unwrap_or(0);
                                playback::play_local_tracks(
                                    runtime.clone(),
                                    weak.clone(),
                                    handle.clone(),
                                    tracks,
                                    start,
                                    false,
                                );
                            }
                        }
                    }
                    "toggle-select" => {
                        if let Some(w) = weak.upgrade() {
                            local_library::toggle_track_select(&w, id.as_str());
                        }
                    }
                    "play-next" | "queue" => {
                        // Resolve the row from the loaded cache (no DB) and
                        // enqueue; folder-detail rows aren't in the Tracks
                        // cache, so fall back to a DB resolve off-thread.
                        let play_next = action.as_str() == "play-next";
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            playback::enqueue_local_tracks(
                                runtime.clone(),
                                handle.clone(),
                                vec![row],
                                play_next,
                            );
                        } else if let Ok(rid) = id.parse::<i64>() {
                            let runtime = runtime.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                if let Some(row) = row {
                                    playback::enqueue_local_tracks(
                                        runtime,
                                        handle2,
                                        vec![row],
                                        play_next,
                                    );
                                }
                            });
                        }
                    }
                    "add-to-playlist" => {
                        // Per-row picker (Tracks tab + folder-detail rows).
                        // Row ids are resolved source-aware at insert, so a folder row
                        // missing from the Tracks cache still works.
                        let Some(w) = weak.upgrade() else { return };
                        let track_ref = match local_library::local_track_by_id(id.as_str()) {
                            Some(row) => local_picker_ref(&row),
                            None => id.to_string(),
                        };
                        playlist_picker::open_multi(&w, &[track_ref], true);
                        let runtime = runtime.clone();
                        let weak2 = weak.clone();
                        handle.spawn(async move {
                            let playlists = playlist_picker::load(&runtime).await;
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                playlist_picker::apply(&w, playlists);
                            });
                        });
                    }
                    "add-to-mixtape" => {
                        // Single-row Add to Mixtape/Collection (Tracks tab +
                        // folder-detail rows; spec §3.1). Same resolution as
                        // play-next: loaded cache first, DB fallback
                        // off-thread for folder rows.
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            let items = myqbz_add::track_items_from_local(&[row]);
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                        } else if let Ok(rid) = id.parse::<i64>() {
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                if let Some(row) = row {
                                    let items = myqbz_add::track_items_from_local(&[row]);
                                    open_add_to_mixtape(weak2, handle2, items);
                                }
                            });
                        }
                    }
                    "favorite" => {
                        // Library-surface favorite: the menu only shows the
                        // entry on qobuz_download rows (TrackRow gates on
                        // source == "qobuz"), and the toggle uses the row's
                        // REAL qobuz_track_id — never the local row id, which
                        // is what Tauri sends (spec §3.2 latent bug; we port
                        // the intent, not the bug).
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            match row.qobuz_track_id {
                                Some(qid) => toggle_track_favorite(
                                    runtime.clone(),
                                    weak.clone(),
                                    handle.clone(),
                                    qid.to_string(),
                                ),
                                None => log::debug!(
                                    "[qbz-slint] favorite: local row {id} has no qobuz_track_id"
                                ),
                            }
                        } else if let Ok(rid) = id.parse::<i64>() {
                            // Folder rows aren't in the Tracks cache: resolve
                            // off-thread, then hop back to the UI thread (the
                            // toggle reads/writes UI models).
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                let Some(qid) = row.and_then(|r| r.qobuz_track_id) else {
                                    log::debug!(
                                        "[qbz-slint] favorite: row {rid} has no qobuz_track_id"
                                    );
                                    return;
                                };
                                let weak3 = weak2.clone();
                                let _ = weak2.upgrade_in_event_loop(move |_w| {
                                    toggle_track_favorite(
                                        runtime,
                                        weak3,
                                        handle2,
                                        qid.to_string(),
                                    );
                                });
                            });
                        }
                    }
                    "go-to-album" | "go-to-artist" => {
                        // Owner improvement over Tauri (which omits both on
                        // local rows): resolve the row (Tracks cache first,
                        // DB fallback for folder-detail rows — same seam as
                        // favorite) and source-route in local_row_goto
                        // (local -> local album view / LocalLibrary
                        // artist by name; qobuz_download -> the REAL Qobuz
                        // pages via its qobuz_track_id).
                        let to_artist = action.as_str() == "go-to-artist";
                        if let Some(row) = local_library::local_track_by_id(id.as_str()) {
                            local_row_goto(runtime.clone(), weak.clone(), &handle, row, to_artist);
                        } else if let Ok(rid) = id.parse::<i64>() {
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let row = tokio::task::spawn_blocking(move || {
                                    crate::library_db::with_db(|db| db.get_track(rid))
                                        .flatten()
                                })
                                .await
                                .ok()
                                .flatten();
                                match row {
                                    Some(row) => local_row_goto(
                                        runtime, weak2, &handle2, row, to_artist,
                                    ),
                                    None => log::debug!(
                                        "[qbz-slint] go-to: local row {rid} not found"
                                    ),
                                }
                            });
                        }
                    }
                    _ => {
                        log::debug!("[qbz-slint] unhandled local track action: {id} {action}");
                    }
                }
            });
    }

    // ---- Tracks tab: sort + group-by + multi-select + bulk ----
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_set_group(move |mode| {
                if let Some(w) = weak.upgrade() {
                    local_library::set_tracks_group(&w, mode.as_str());
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_set_sort(move |key| {
                if let Some(w) = weak.upgrade() {
                    local_library::set_tracks_sort(&w, key.as_str(), handle.clone());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_toggle_multi_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<LocalLibraryState>().get_tracks_multi_select();
                    local_library::set_tracks_multi_select(&w, !on);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_tracks_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => local_library::select_all_tracks(&w),
                    "clear" => local_library::clear_tracks_selection(&w),
                    "queue" => {
                        let rows = local_library::selected_local_tracks(&w);
                        playback::enqueue_local_tracks(runtime.clone(), handle.clone(), rows, false);
                        local_library::clear_tracks_selection(&w);
                    }
                    "play-next" => {
                        let rows = local_library::selected_local_tracks(&w);
                        playback::enqueue_local_tracks(runtime.clone(), handle.clone(), rows, true);
                        local_library::clear_tracks_selection(&w);
                    }
                    "add-to-playlist" => {
                        // Source-aware refs: library row ids (resolved at insert).
                        let rows = local_library::selected_local_tracks(&w);
                        let ids: Vec<String> = rows.iter().map(local_picker_ref).collect();
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, true);
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak2.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    "add-to-mixtape" => {
                        // All selected tracks.
                        let rows = local_library::selected_local_tracks(&w);
                        let items = myqbz_add::track_items_from_local(&rows);
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            local_library::clear_tracks_selection(&w);
                        }
                    }
                    _ => {}
                }
            });
    }
    {
        // Albums-grid multi-select toggle.
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_albums_toggle_multi_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<LocalLibraryState>().get_albums_multi_select();
                    local_library::set_albums_multi_select(&w, !on);
                }
            });
    }
    {
        // Albums-grid bulk bar. Album->tracks resolution is a blocking DB read
        // (fetch_album_tracks_blocking), so it runs on spawn_blocking; the
        // resulting LocalTracks feed the same enqueue/playlist/mixtape paths as
        // the Tracks tab.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_albums_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => local_library::select_all_albums(&w),
                    "clear" => local_library::clear_albums_selection(&w),
                    "queue" | "play-next" => {
                        let keys = local_library::selected_album_ids(&w);
                        let play_next = action.as_str() == "play-next";
                        let runtime = runtime.clone();
                        let handle2 = handle.clone();
                        handle.spawn(async move {
                            let rows = tokio::task::spawn_blocking(move || {
                                local_library::selected_albums_tracks_blocking(&keys)
                            })
                            .await
                            .unwrap_or_default();
                            playback::enqueue_local_tracks(runtime, handle2, rows, play_next);
                        });
                        local_library::clear_albums_selection(&w);
                    }
                    "add-to-playlist" => {
                        let keys = local_library::selected_album_ids(&w);
                        if !keys.is_empty() {
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            handle.spawn(async move {
                                let rows = tokio::task::spawn_blocking(move || {
                                    local_library::selected_albums_tracks_blocking(&keys)
                                })
                                .await
                                .unwrap_or_default();
                                let ids: Vec<String> = rows.iter().map(local_picker_ref).collect();
                                let runtime2 = runtime.clone();
                                let _ = weak2.upgrade_in_event_loop(move |w| {
                                    if !ids.is_empty() {
                                        playlist_picker::open_multi(&w, &ids, true);
                                    }
                                });
                                let playlists = playlist_picker::load(&runtime2).await;
                                let _ = weak2.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    "add-to-mixtape" => {
                        let keys = local_library::selected_album_ids(&w);
                        if !keys.is_empty() {
                            let weak2 = weak.clone();
                            let handle2 = handle.clone();
                            handle.spawn(async move {
                                let rows = tokio::task::spawn_blocking(move || {
                                    local_library::selected_albums_tracks_blocking(&keys)
                                })
                                .await
                                .unwrap_or_default();
                                let _ = weak2.upgrade_in_event_loop(move |w| {
                                    let items = myqbz_add::track_items_from_local(&rows);
                                    if !items.is_empty() {
                                        open_add_to_mixtape(w.as_weak(), handle2, items);
                                        local_library::clear_albums_selection(&w);
                                    }
                                });
                            });
                        }
                    }
                    _ => {}
                }
            });
    }

    // ---- Folders tree rail: search / collapse / multi-select ----
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_tree_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    local_library::folders_tree_search(&w, query.as_str());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_collapse_all(move || {
                if let Some(w) = weak.upgrade() {
                    local_library::collapse_all_tree(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_toggle_select_mode(move || {
                if let Some(w) = weak.upgrade() {
                    local_library::toggle_tree_select_mode(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_toggle_folder_select(move |path| {
                local_library::toggle_tree_folder_select(weak.clone(), handle.clone(), path.to_string());
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_toggle_track_select(move |path| {
                local_library::toggle_tree_track_select(weak.clone(), handle.clone(), path.to_string());
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => {
                        local_library::tree_select_all(weak.clone(), handle.clone());
                    }
                    "clear" => local_library::tree_clear_selection(&w),
                    "queue" => {
                        let rows = local_library::tree_selected_snapshot();
                        playback::enqueue_local_tracks(runtime.clone(), handle.clone(), rows, false);
                        local_library::tree_clear_selection(&w);
                    }
                    "play-next" => {
                        let rows = local_library::tree_selected_snapshot();
                        playback::enqueue_local_tracks(runtime.clone(), handle.clone(), rows, true);
                        local_library::tree_clear_selection(&w);
                    }
                    "add-to-playlist" => {
                        // Source-aware refs (library row ids).
                        let rows = local_library::tree_selected_snapshot();
                        let ids: Vec<String> = rows.iter().map(local_picker_ref).collect();
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, true);
                            let runtime = runtime.clone();
                            let weak2 = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak2.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    "add-to-mixtape" => {
                        // All selected tracks.
                        let rows = local_library::tree_selected_snapshot();
                        let items = myqbz_add::track_items_from_local(&rows);
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            local_library::tree_clear_selection(&w);
                        }
                    }
                    _ => {}
                }
            });
    }

    // ---- Folders tab actions ----
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_search(move |_query| {
                // Query is two-way bound to folders-search; re-derive in place.
                if let Some(w) = weak.upgrade() {
                    local_library::derive_folders(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_set_sort(move |sort| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_folders_sort(sort);
                    local_library::derive_folders(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folders_set_group(move |group| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>().set_folders_group(group);
                    local_library::derive_folders(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_set_mode(move |mode| {
                if let Some(w) = weak.upgrade() {
                    w.global::<LocalLibraryState>()
                        .set_folders_view_mode(mode.clone());
                }
                // Lazy-load the tree roots the first time tree mode is shown.
                if mode.as_str() == "tree" {
                    local_library::ensure_folder_tree_loaded(weak.clone(), handle.clone());
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_toggle_node(move |path, expand| {
                local_library::toggle_folder_node(
                    weak.clone(),
                    handle.clone(),
                    path.to_string(),
                    expand,
                );
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_select(move |path, segment| {
                local_library::select_folder(
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                    path.to_string(),
                    segment.to_string(),
                );
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_folder_detail_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    local_library::folder_detail_search(&w, query.as_str());
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_play_node(move |path| {
                playback::play_local_folder_recursive(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    path.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_folders_play_track(move |id| {
                if let Ok(row_id) = id.parse::<i64>() {
                    let path = weak
                        .upgrade()
                        .map(|w| {
                            w.global::<LocalLibraryState>()
                                .get_folders_selected_path()
                                .to_string()
                        })
                        .unwrap_or_default();
                    if !path.is_empty() {
                        playback::play_local_folder_tracks_from(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            path,
                            row_id,
                        );
                    }
                }
            });
    }

    // ---- Ephemeral folder actions ----
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_open(move || {
                local_library::open_ephemeral(runtime.clone(), weak.clone(), handle.clone());
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_play_all(move || {
                playback::ephemeral_play_or_prompt(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    "all".to_string(),
                    String::new(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_play_track(move |id| {
                playback::ephemeral_play_or_prompt(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    "track".to_string(),
                    id.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_play_album(move |key| {
                playback::ephemeral_play_or_prompt(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    "album".to_string(),
                    key.to_string(),
                );
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<LocalLibraryActions>()
            .on_ephemeral_clear(move || {
                let runtime = runtime.clone();
                let weak = weak.clone();
                handle.spawn(async move {
                    // Stop a playing ephemeral track before dropping the session
                    // so its (about-to-be-reused) id can't false-highlight rows.
                    playback::wipe_ephemeral_if_playing(&runtime, &weak).await;
                    let _ = weak.upgrade_in_event_loop(|w| {
                        local_library::clear_ephemeral(&w);
                    });
                });
            });
    }
    // Ephemeral "already playing" choice modal — clear-and-play vs add-to-queue.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<EphemeralPlayChoiceActions>()
            .on_replace(move || {
                if let Some(w) = weak.upgrade() {
                    let s = w.global::<EphemeralPlayChoiceState>();
                    let kind = s.get_intent_kind().to_string();
                    let arg = s.get_intent_arg().to_string();
                    s.set_open(false);
                    playback::ephemeral_play(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        kind,
                        arg,
                    );
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<EphemeralPlayChoiceActions>()
            .on_enqueue(move || {
                if let Some(w) = weak.upgrade() {
                    let s = w.global::<EphemeralPlayChoiceState>();
                    let kind = s.get_intent_kind().to_string();
                    let arg = s.get_intent_arg().to_string();
                    s.set_open(false);
                    playback::ephemeral_enqueue(
                        runtime.clone(),
                        weak.clone(),
                        handle.clone(),
                        kind,
                        arg,
                    );
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<EphemeralPlayChoiceActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<EphemeralPlayChoiceState>().set_open(false);
                }
            });
    }

    // Restore a previously-open ephemeral folder (re-scans the path; does NOT
    // switch the landing view). Runs once at startup.
    local_library::rehydrate_ephemeral(window.as_weak(), tokio_rt.handle().clone());

    // ---- Artists tab actions ----
    {
        let weak = window.as_weak();
        window
            .global::<LocalLibraryActions>()
            .on_artists_search(move |_query| {
                // Query is two-way bound to artists-search; re-derive in place.
                if let Some(w) = weak.upgrade() {
                    local_library::derive_artists(&w);
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<LocalLibraryActions>()
            .on_artists_select(move |name| {
                local_library::select_local_artist(
                    weak.clone(),
                    handle.clone(),
                    image_cache.clone(),
                    name.to_string(),
                );
            });
    }

    // Discover "View all" — open the full-list page for a section,
    // recording it as a history entry (mirrors the favorites branch
    // of on_header_menu_navigate).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.on_discover_view_all(move |endpoint, title| {
            nav::record(nav::NavEntry::DiscoverBrowse {
                endpoint: endpoint.to_string(),
                title: title.to_string(),
            });
            if let Some(w) = weak.upgrade() {
                update_nav_flags(&w);
            }
            discover_browse::navigate(
                runtime.clone(),
                weak.clone(),
                &handle,
                image_cache.clone(),
                endpoint.to_string(),
                title.to_string(),
                current_genre_filter(),
            );
        });
    }

    // Discover "View all" pagination — load the next album page when
    // the grid scrolls near the bottom.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<DiscoverBrowseActions>()
            .on_load_more(move || {
                discover_browse::load_more(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    current_genre_filter(),
                );
            });
    }

    // Discover "View all" search — re-filter the loaded albums
    // client-side after the search box changes (UI thread).
    {
        let weak = window.as_weak();
        window
            .global::<DiscoverBrowseActions>()
            .on_search_changed(move || {
                if let Some(w) = weak.upgrade() {
                    discover_browse::apply_filter(&w);
                }
            });
    }

    // Qobuz Playlists "View all" — pagination, client-side search and the
    // single-select category tag bar (server-side re-fetch from offset 0).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<PlaylistBrowseActions>()
            .on_load_more(move || {
                playlist_browse::load_more(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    current_genre_filter(),
                );
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistBrowseActions>()
            .on_search_changed(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_browse::apply_filter(&w);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<PlaylistBrowseActions>()
            .on_select_tag(move |slug| {
                playlist_browse::select_tag(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    slug.to_string(),
                    current_genre_filter(),
                );
            });
    }

    // Favorites view actions — tab switch (lazy-load), open album /
    // artist, and per-row track actions routed to the media-action
    // "Add to playlist" picker — pick TOGGLES membership (checkbox
    // semantics, spec PLAYLIST-REDESIGN-SPEC.md §4): not-yet-present adds
    // the pending track(s), already-present removes them. Never closes the
    // picker (only close() does — footer "Done" / backdrop); close
    // dismisses.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<PlaylistPickerActions>()
            .on_pick(move |playlist_id| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let picker = w.global::<PlaylistPickerState>();
                let is_local = picker.get_local_mode();
                // Bulk add carries track-ids; single add carries track-id.
                let ids_model = picker.get_track_ids();
                let track_id_single = picker.get_track_id().to_string();
                // Resolve the target name for the success toast.
                let target_name = picker_playlist_name(&w, playlist_id.as_str());

                let already_has = {
                    use slint::Model;
                    let model = picker.get_playlists();
                    (0..model.row_count())
                        .filter_map(|i| model.row_data(i))
                        .find(|item| item.id.as_str() == playlist_id.as_str())
                        .map(|item| item.already_has)
                        .unwrap_or(false)
                };
                if already_has {
                    toggle_off_playlist_pick(
                        &runtime,
                        &weak,
                        &handle,
                        playlist_id.to_string(),
                        target_name,
                        is_local,
                        &ids_model,
                        &track_id_single,
                    );
                    return;
                }

                // --- ADD (unchanged below except the row is no longer
                // closed on pick — see toggle_off_playlist_pick for the
                // remove side) ---
                // LOCAL playlist target (id "local:<uuid>") — writes go to
                // the library.db repo (works offline; D7 routing).
                if local_playlist::is_local_id(playlist_id.as_str()) {
                    let target = playlist_id.to_string();
                    if is_local {
                        // Local-mode refs — LocalLibrary row ids ("<i64>",
                        // source-aware mapping: local path / offline-copy
                        // Qobuz id).
                        let refs: Vec<String> = (0..ids_model.row_count())
                            .filter_map(|i| ids_model.row_data(i))
                            .map(|s| s.to_string())
                            .collect();
                        if refs.is_empty() {
                            return;
                        }
                        let weak = weak.clone();
                        let tname = target_name.clone();
                        let mark_id = target.clone();
                        handle.spawn(async move {
                            let added = tokio::task::spawn_blocking(move || {
                                local_playlist::add_local_refs_blocking(&target, &refs)
                            })
                            .await
                            .unwrap_or(0);
                            // reco: local refs are not Qobuz catalog ids — not
                            // logged (same source gate as local plays).
                            toast_added_tracks(&weak, added, tname);
                            if added > 0 {
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::mark_row_already_has(&w, &mark_id, true);
                                });
                            }
                        });
                        return;
                    }
                    let mut ids: Vec<u64> = (0..ids_model.row_count())
                        .filter_map(|i| ids_model.row_data(i))
                        .filter_map(|s| s.parse::<u64>().ok())
                        .collect();
                    if ids.is_empty() {
                        if let Ok(tid) = track_id_single.parse::<u64>() {
                            ids.push(tid);
                        }
                    }
                    if ids.is_empty() {
                        return;
                    }
                    let weak = weak.clone();
                    let tname = target_name.clone();
                    let mark_id = target.clone();
                    handle.spawn(async move {
                        // reco: keep the full Qobuz ids before they move into
                        // the add closure (local-playlist target = no Qobuz pid).
                        let reco_ids = ids.clone();
                        let added = tokio::task::spawn_blocking(move || {
                            local_playlist::add_qobuz_tracks_blocking(&target, &ids)
                        })
                        .await
                        .unwrap_or(0);
                        tokio::task::spawn_blocking(move || {
                            crate::reco::log_playlist_add(None, reco_ids)
                        });
                        toast_added_tracks(&weak, added, tname);
                        if added > 0 {
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                playlist_picker::mark_row_already_has(&w, &mark_id, true);
                            });
                        }
                    });
                    return;
                }

                let Ok(pid) = playlist_id.parse::<u64>() else {
                    return;
                };

                if is_local {
                    // Local-mode refs onto a QOBUZ playlist: row ids attach
                    // via the local sidecar (same table the offline detail
                    // renders).
                    let refs: Vec<String> = (0..ids_model.row_count())
                        .filter_map(|i| ids_model.row_data(i))
                        .map(|s| s.to_string())
                        .collect();
                    if refs.is_empty() {
                        return;
                    }
                    // Seam C: append after the whole merged list AND past
                    // any stored position (the old 0-based `enumerate`
                    // write collided slots -> silent row loss in the
                    // interleave). Base = the Qobuz block size from the
                    // sidebar's session cache; re-adding an existing ref
                    // MOVES it to the append slot (INSERT OR REPLACE, E4).
                    let qobuz_count = sidebar::playlist_track_count(pid).unwrap_or(0);
                    let refs_count = refs.len();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle2 = handle.clone();
                    let image_cache = image_cache.clone();
                    let tname = target_name.clone();
                    handle.spawn(async move {
                        let _ = tokio::task::spawn_blocking(move || {
                            crate::library_db::with_db(|db| {
                                let mut next =
                                    db.next_playlist_sidecar_position(pid, qobuz_count)?;
                                for r in refs.iter() {
                                    if let Ok(lid) = r.parse::<i64>() {
                                        db.add_local_track_to_playlist(pid, lid, next)?;
                                        next += 1;
                                    }
                                }
                                Ok(())
                            })
                        })
                        .await;
                        // reco: local refs are not Qobuz catalog ids — not
                        // logged (same source gate as local plays).
                        toast_added_tracks(&weak, refs_count, tname);
                        if refs_count > 0 {
                            let _ = weak.clone().upgrade_in_event_loop(move |w| {
                                playlist_picker::mark_row_already_has(&w, &pid.to_string(), true);
                            });
                        }
                        // E12: the open detail re-merges so the rows show
                        // up immediately.
                        let _ = weak.clone().upgrade_in_event_loop(move |w| {
                            if w.global::<NavState>().get_view() == ContentView::Playlist
                                && w.global::<PlaylistState>().get_id().to_string()
                                    == pid.to_string()
                            {
                                navigate_playlist(
                                    runtime,
                                    weak,
                                    &handle2,
                                    image_cache,
                                    pid.to_string(),
                                );
                            }
                        });
                    });
                    return;
                }

                // Qobuz tracks → Qobuz playlist. Run duplicate detection first
                // (Tauri parity: this is the ONLY branch that checks dupes).
                // If any of the ids are already in the target, park the context
                // in DUP_CONFIRM_STASH and open the confirm sub-modal; the user
                // then chooses add-all / add-new-only. With no dupes we add
                // directly and toast.
                let mut ids: Vec<u64> = (0..ids_model.row_count())
                    .filter_map(|i| ids_model.row_data(i))
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect();
                if ids.is_empty() {
                    if let Ok(tid) = track_id_single.parse::<u64>() {
                        ids.push(tid);
                    }
                }
                if ids.is_empty() {
                    return;
                }
                let runtime = runtime.clone();
                let weak = weak.clone();
                let tname = target_name.clone();
                handle.spawn(async move {
                    match runtime.core().check_playlist_duplicates(pid, &ids).await {
                        Ok(dup) if dup.duplicate_count > 0 => {
                            // Stash the full context; the confirm handlers read
                            // it back. Open the sub-modal with the counts.
                            let total = dup.total_tracks as i32;
                            let dupc = dup.duplicate_count as i32;
                            let dup_ids = dup.duplicate_track_ids.clone();
                            let stash = (pid, ids.clone(), dup_ids, tname.clone());
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                DUP_CONFIRM_STASH.with(|c| *c.borrow_mut() = Some(stash));
                                let st = w.global::<DuplicateConfirmState>();
                                st.set_duplicate_count(dupc);
                                st.set_total_count(total);
                                st.set_busy(false);
                                st.set_playlist_name(tname.into());
                                st.set_open(true);
                            });
                        }
                        Ok(_) => {
                            // No duplicates — add directly + toast.
                            let n = ids.len();
                            if let Err(e) =
                                runtime.core().add_tracks_to_playlist(pid, &ids).await
                            {
                                log::error!("[qbz-slint] add to playlist failed: {e}");
                            } else {
                                // reco: log the full requested Qobuz ids.
                                let reco_ids = ids.clone();
                                tokio::task::spawn_blocking(move || {
                                    crate::reco::log_playlist_add(Some(pid), reco_ids)
                                });
                                toast_added_tracks(&weak, n, tname);
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::mark_row_already_has(&w, &pid.to_string(), true);
                                });
                            }
                        }
                        Err(e) => {
                            // Dup check failed (e.g. transient API) — fall back
                            // to a direct add so the action still completes.
                            log::warn!(
                                "[qbz-slint] dup check failed, adding directly: {e}"
                            );
                            let n = ids.len();
                            if let Err(e) =
                                runtime.core().add_tracks_to_playlist(pid, &ids).await
                            {
                                log::error!("[qbz-slint] add to playlist failed: {e}");
                            } else {
                                // reco: log the full requested Qobuz ids.
                                let reco_ids = ids.clone();
                                tokio::task::spawn_blocking(move || {
                                    crate::reco::log_playlist_add(Some(pid), reco_ids)
                                });
                                toast_added_tracks(&weak, n, tname);
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::mark_row_already_has(&w, &pid.to_string(), true);
                                });
                            }
                        }
                    }
                });
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistPickerActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<PlaylistPickerState>();
                    st.set_open(false);
                    // Reset the inline-create + filter affordances so the next
                    // open starts clean.
                    st.set_creating_open(false);
                    st.set_create_name("".into());
                    st.set_creating(false);
                    st.set_filter("".into());
                }
            });
    }

    // Inline "Create new playlist" → create-and-add (PlaylistCreateRow).
    // Creates a playlist (Qobuz online / local offline per D8) and adds the
    // carried tracks to it, collapses the create row, and reloads the
    // picker list so the new playlist shows up checked — the picker itself
    // STAYS OPEN (spec §2/§4: only "Done" / backdrop close it). Discriminates
    // the carried ids exactly like the pick handler (local-mode refs vs
    // Qobuz u64 ids).
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistPickerActions>()
            .on_create_and_add(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                use slint::Model;
                let picker = w.global::<PlaylistPickerState>();
                let name = picker.get_create_name().to_string();
                if name.trim().is_empty() || picker.get_creating() {
                    return;
                }
                let is_local = picker.get_local_mode();
                let ids_model = picker.get_track_ids();
                let track_id_single = picker.get_track_id().to_string();
                // Local-mode refs (LocalLibrary row ids) for the
                // local-playlist add; Qobuz u64 ids for the online path.
                let refs: Vec<String> = (0..ids_model.row_count())
                    .filter_map(|i| ids_model.row_data(i))
                    .map(|s| s.to_string())
                    .collect();
                let mut qobuz_ids: Vec<u64> = (0..ids_model.row_count())
                    .filter_map(|i| ids_model.row_data(i))
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect();
                if qobuz_ids.is_empty() {
                    if let Ok(tid) = track_id_single.parse::<u64>() {
                        qobuz_ids.push(tid);
                    }
                }
                picker.set_creating(true);

                let offline_now = offline_mode::engine().is_offline();
                let nm = name.trim().to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle2 = handle.clone();

                if offline_now {
                    // D8: offline ⇒ LOCAL playlist (library.db), never the
                    // retired pending-playlist engine. Mirrors the create
                    // modal's offline branch.
                    let local_refs = refs.clone();
                    let local_qobuz = qobuz_ids.clone();
                    // reco: the full Qobuz ids (empty when adding local refs).
                    let reco_qobuz: Vec<u64> = if is_local { Vec::new() } else { qobuz_ids.clone() };
                    handle.spawn(async move {
                        let created = tokio::task::spawn_blocking({
                            let nm = nm.clone();
                            move || local_playlist::create_blocking(&nm, None, true)
                        })
                        .await
                        .ok()
                        .flatten();
                        let mut added = 0usize;
                        if let Some(ref new_id) = created {
                            let new_id = new_id.clone();
                            added = tokio::task::spawn_blocking(move || {
                                if is_local {
                                    local_playlist::add_local_refs_blocking(
                                        &new_id,
                                        &local_refs,
                                    )
                                } else {
                                    local_playlist::add_qobuz_tracks_blocking(
                                        &new_id,
                                        &local_qobuz,
                                    )
                                }
                            })
                            .await
                            .unwrap_or(0);
                        }
                        // reco: log the new playlist's Qobuz tracks (new local
                        // playlist = no Qobuz pid; empty when local refs).
                        if created.is_some() {
                            let reco_ids = reco_qobuz;
                            tokio::task::spawn_blocking(move || {
                                crate::reco::log_playlist_add(None, reco_ids)
                            });
                        }
                        let r2 = runtime.clone();
                        let h2 = handle2.clone();
                        let weak2 = weak.clone();
                        let nm2 = nm.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            let st = w.global::<PlaylistPickerState>();
                            st.set_creating(false);
                            st.set_creating_open(false);
                            st.set_create_name("".into());
                            // Stays open (spec §2: only the footer "Done" /
                            // backdrop close it) — reload so the new playlist
                            // appears, checked if tracks were carried into it.
                            match created {
                                Some(_) => {
                                    toast_added_tracks(&weak2, added, nm2);
                                    load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                    h2.spawn(async move {
                                        let playlists = playlist_picker::load(&r2).await;
                                        let _ = weak2.upgrade_in_event_loop(move |w| {
                                            playlist_picker::apply(&w, playlists)
                                        });
                                    });
                                }
                                None => {
                                    log::error!(
                                        "[qbz-slint] create-and-add (local) failed"
                                    );
                                }
                            }
                        });
                    });
                    return;
                }

                // Online ⇒ Qobuz playlist, then add the carried tracks.
                handle.spawn(async move {
                    match runtime.core().create_playlist(&nm, None, false).await {
                        Ok(playlist) => {
                            let pid = playlist.id;
                            let n = qobuz_ids.len();
                            if !qobuz_ids.is_empty() {
                                if let Err(e) = runtime
                                    .core()
                                    .add_tracks_to_playlist(pid, &qobuz_ids)
                                    .await
                                {
                                    log::error!(
                                        "[qbz-slint] create-and-add: add failed: {e}"
                                    );
                                }
                                // reco: log the new playlist's Qobuz tracks.
                                let reco_ids = qobuz_ids.clone();
                                tokio::task::spawn_blocking(move || {
                                    crate::reco::log_playlist_add(Some(pid), reco_ids)
                                });
                            }
                            let r2 = runtime.clone();
                            let h2 = handle2.clone();
                            let weak2 = weak.clone();
                            let nm2 = nm.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                let st = w.global::<PlaylistPickerState>();
                                st.set_creating(false);
                                st.set_creating_open(false);
                                st.set_create_name("".into());
                                // Stays open — see the offline branch above.
                                toast_added_tracks(&weak2, n, nm2);
                                load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                h2.spawn(async move {
                                    let playlists = playlist_picker::load(&r2).await;
                                    let _ = weak2
                                        .upgrade_in_event_loop(move |w| playlist_picker::apply(&w, playlists));
                                });
                            });
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] create-and-add: create failed: {e}");
                            let _ = weak.upgrade_in_event_loop(|w| {
                                w.global::<PlaylistPickerState>().set_creating(false);
                            });
                        }
                    }
                });
            });
    }

    // Picker client-side filter — recompute each row's `filter-rank`
    // (case-insensitive substring; Slint 1.16 has no string `.contains`, so
    // the match runs here). Rank = match ordinal among the filtered rows,
    // -1 = filtered out; the total lands in `filter-matches`. Pure frontend,
    // no backend call.
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistPickerActions>()
            .on_filter_changed(move |query| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                use slint::Model;
                let needle = query.to_lowercase();
                let model = w.global::<PlaylistPickerState>().get_playlists();
                let mut rank: i32 = 0;
                for i in 0..model.row_count() {
                    if let Some(mut item) = model.row_data(i) {
                        let matches = needle.is_empty()
                            || item.name.to_lowercase().contains(&needle);
                        let new_rank = if matches { rank } else { -1 };
                        if matches {
                            rank += 1;
                        }
                        if item.filter_rank != new_rank {
                            item.filter_rank = new_rank;
                            model.set_row_data(i, item);
                        }
                    }
                }
                w.global::<PlaylistPickerState>().set_filter_matches(rank);
            });
    }

    // Duplicate-confirm sub-modal handlers. The pending context lives in
    // DUP_CONFIRM_STASH (set by the picker's Qobuz→Qobuz branch). Each handler
    // reads it, performs the chosen add, toasts, then closes + clears.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<DuplicateConfirmActions>()
            .on_add_all(move || {
                let Some(stash) = DUP_CONFIRM_STASH.with(|c| c.borrow_mut().take()) else {
                    return;
                };
                let (pid, all_ids, _dup_ids, name) = stash;
                if let Some(w) = weak.upgrade() {
                    w.global::<DuplicateConfirmState>().set_busy(true);
                }
                let runtime = runtime.clone();
                let weak = weak.clone();
                handle.spawn(async move {
                    let n = all_ids.len();
                    if let Err(e) = runtime.core().add_tracks_to_playlist(pid, &all_ids).await
                    {
                        log::error!("[qbz-slint] dup add-all failed: {e}");
                    } else {
                        // reco: log the full requested Qobuz ids (add-all).
                        let reco_ids = all_ids.clone();
                        tokio::task::spawn_blocking(move || {
                            crate::reco::log_playlist_add(Some(pid), reco_ids)
                        });
                        toast_added_tracks(&weak, n, name);
                    }
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        let st = w.global::<DuplicateConfirmState>();
                        st.set_busy(false);
                        st.set_open(false);
                        playlist_picker::mark_row_already_has(&w, &pid.to_string(), true);
                    });
                });
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<DuplicateConfirmActions>()
            .on_add_new_only(move || {
                let Some(stash) = DUP_CONFIRM_STASH.with(|c| c.borrow_mut().take()) else {
                    return;
                };
                let (pid, all_ids, dup_ids, name) = stash;
                // reco: keep the FULL requested ids before the dedup consumes
                // them (Tauri logs the full request, not the inserted subset).
                let reco_all = all_ids.clone();
                // Add only the non-duplicate ids (all \ duplicates). If nothing
                // is left, just close.
                let to_add: Vec<u64> =
                    all_ids.into_iter().filter(|id| !dup_ids.contains(id)).collect();
                if to_add.is_empty() {
                    if let Some(w) = weak.upgrade() {
                        w.global::<DuplicateConfirmState>().set_open(false);
                    }
                    return;
                }
                if let Some(w) = weak.upgrade() {
                    w.global::<DuplicateConfirmState>().set_busy(true);
                }
                let runtime = runtime.clone();
                let weak = weak.clone();
                handle.spawn(async move {
                    let n = to_add.len();
                    if let Err(e) = runtime.core().add_tracks_to_playlist(pid, &to_add).await
                    {
                        log::error!("[qbz-slint] dup add-new-only failed: {e}");
                    } else {
                        // reco: log the FULL requested ids (Tauri parity), not
                        // just the non-duplicate subset that was inserted.
                        let reco_ids = reco_all;
                        tokio::task::spawn_blocking(move || {
                            crate::reco::log_playlist_add(Some(pid), reco_ids)
                        });
                        toast_added_tracks(&weak, n, name);
                    }
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        let st = w.global::<DuplicateConfirmState>();
                        st.set_busy(false);
                        st.set_open(false);
                        playlist_picker::mark_row_already_has(&w, &pid.to_string(), true);
                    });
                });
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<DuplicateConfirmActions>()
            .on_cancel(move || {
                DUP_CONFIRM_STASH.with(|c| *c.borrow_mut() = None);
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<DuplicateConfirmState>();
                    st.set_busy(false);
                    st.set_open(false);
                }
            });
    }

    // Track drag onto sidebar playlists (a star QBZ feature).
    {
        let weak = window.as_weak();
        window.global::<DragActions>().on_start(
            move |track_id, title, subtitle, gx, gy| {
                let Some(w) = weak.upgrade() else { return };
                log::info!("[qbz-slint][drag] start gx={gx} gy={gy} (cursor should be here)");
                let tracks = gather_drag_tracks(&w, track_id.as_str());
                let count = tracks.len();
                drag::set_dragged(tracks);
                let ds = w.global::<DragState>();
                ds.set_count(count as i32);
                ds.set_ghost_title(title);
                ds.set_ghost_subtitle(subtitle);
                ds.set_pointer_x(gx);
                ds.set_pointer_y(gy);
                ds.set_over_playlist_id("".into());
                ds.set_active(true);
            },
        );
    }
    {
        let weak = window.as_weak();
        window.global::<DragActions>().on_move(move |gx, gy| {
            if let Some(w) = weak.upgrade() {
                let ds = w.global::<DragState>();
                ds.set_pointer_x(gx);
                ds.set_pointer_y(gy);
            }
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<DragActions>().on_end(move || {
            let Some(w) = weak.upgrade() else { return };
            let ds = w.global::<DragState>();
            let pid = ds.get_over_playlist_id().to_string();
            ds.set_active(false);
            ds.set_over_playlist_id("".into());
            let tracks = drag::dragged();
            drag::clear();
            if tracks.is_empty() {
                return;
            }
            // Drop onto a LOCAL playlist row — write the repo source-aware
            // (D7 routing): local file rows store local_path,
            // Qobuz/offline-cached rows qobuz_track_id.
            if local_playlist::is_local_id(&pid) {
                handle.spawn(async move {
                    let n = tokio::task::spawn_blocking(move || {
                        local_playlist::add_drag_tracks_blocking(&pid, &tracks)
                    })
                    .await
                    .unwrap_or(0);
                    log::info!("[qbz-slint] dropped {n} track(s) onto local playlist");
                });
                return;
            }
            if let Ok(pid) = pid.parse::<u64>() {
                // Qobuz playlist target: catalog ids become real membership;
                // local rows attach via the mixed-playlist sidecar (the
                // same table the picker's local mode writes).
                let mut qobuz: Vec<u64> = Vec::new();
                let mut local_rows: Vec<i64> = Vec::new();
                for item in tracks {
                    match item {
                        drag::DragTrack::Qobuz(id) => qobuz.push(id),
                        drag::DragTrack::LocalRow(id) => local_rows.push(id),
                    }
                }
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle2 = handle.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    let mut added = 0usize;
                    if !qobuz.is_empty() {
                        match runtime.core().add_tracks_to_playlist(pid, &qobuz).await {
                            Ok(()) => added += qobuz.len(),
                            Err(e) => {
                                log::error!("[qbz-slint] drop add to playlist failed: {e}")
                            }
                        }
                    }
                    let sidecar_added = !local_rows.is_empty();
                    if sidecar_added {
                        // Seam C: append after the merged list / past any
                        // stored position — never 0-based. The Qobuz block
                        // size includes the rows the SAME drop just added
                        // (the sidebar cache hasn't seen them yet).
                        let qobuz_count = sidebar::playlist_track_count(pid)
                            .unwrap_or(0)
                            + qobuz.len() as u32;
                        let n = tokio::task::spawn_blocking(move || {
                            crate::library_db::with_db(|db| {
                                let mut next =
                                    db.next_playlist_sidecar_position(pid, qobuz_count)?;
                                for rid in local_rows.iter() {
                                    db.add_local_track_to_playlist(pid, *rid, next)?;
                                    next += 1;
                                }
                                Ok(local_rows.len())
                            })
                            .unwrap_or(0)
                        })
                        .await
                        .unwrap_or(0);
                        added += n;
                    }
                    if added > 0 {
                        log::info!(
                            "[qbz-slint] dropped {added} track(s) onto playlist {pid}"
                        );
                    }
                    if sidecar_added {
                        // E12: re-merge the open detail after a sidecar
                        // write to the same playlist.
                        let _ = weak.clone().upgrade_in_event_loop(move |w| {
                            if w.global::<NavState>().get_view() == ContentView::Playlist
                                && w.global::<PlaylistState>().get_id().to_string()
                                    == pid.to_string()
                            {
                                navigate_playlist(
                                    runtime,
                                    weak,
                                    &handle2,
                                    image_cache,
                                    pid.to_string(),
                                );
                            }
                        });
                    }
                });
            }
        });
    }

    // Playlist in-page track search (client-side filter).
    {
        let weak = window.as_weak();
        window
            .global::<PlaylistActions>()
            .on_search(move |query| {
                if let Some(w) = weak.upgrade() {
                    playlist::filter_tracks(&w, query.as_str());
                }
            });
    }
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistActions>()
            .on_set_sort(move |field| {
                let Some(w) = weak.upgrade() else { return; };
                playlist::set_sort(&w, field.as_str());
                // Entering custom: load (or seed) the local order, then
                // re-render. Off-thread (opens library.db).
                if field.as_str() == "custom" {
                    let pid = w.global::<PlaylistState>().get_id().to_string();
                    if let Ok(pid) = pid.parse::<u64>() {
                        // Seed keys carry (id, is_local) — Qobuz rows then
                        // local sidecar rows (Tauri parity).
                        let seed = playlist::custom_seed_keys();
                        let weak = weak.clone();
                        handle.spawn(async move {
                            let orders = tokio::task::spawn_blocking(move || {
                                playlist::load_or_init_custom(pid, seed)
                            })
                            .await
                            .unwrap_or_default();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                playlist::apply_custom_order(&w, orders);
                            });
                        });
                    }
                }
            });
    }
    // Drag-reorder within the custom-order track list (issue #589): the
    // drop commits ONE from->to move. Routes like the move-up/move-down
    // chevron arms: LOCAL playlists write the repo position order directly
    // (repo::reorder, B2); Qobuz playlists rebuild the custom-order sidecar
    // optimistically and persist the full order off-thread.
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<PlaylistActions>()
            .on_reorder_track(move |from, to| {
                let Some(w) = weak.upgrade() else { return; };
                if from < 0 || to < 0 || to == from || to == from + 1 {
                    return;
                }
                let (from, to) = (from as usize, to as usize);
                let pid = w.global::<PlaylistState>().get_id().to_string();
                if local_playlist::is_local_id(&pid) {
                    local_playlist::reorder_row(&w, &handle, from, to);
                } else {
                    let orders = playlist::reorder_track(&w, from, to);
                    if !orders.is_empty() {
                        if let Ok(pid) = pid.parse::<u64>() {
                            handle.spawn(async move {
                                tokio::task::spawn_blocking(move || {
                                    playlist::persist_custom(pid, orders);
                                })
                                .await
                                .ok();
                            });
                        }
                    }
                }
            });
    }

    // Edit playlist (rename / delete).
    {
        let weak = window.as_weak();
        window
            .global::<EditPlaylistActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<EditPlaylistState>().set_open(false);
                }
            });
    }
    {
        // Rename the playlist, then refresh the detail header + sidebar.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<EditPlaylistActions>()
            .on_save(move || {
                let Some(w) = weak.upgrade() else { return; };
                let es = w.global::<EditPlaylistState>();
                let name = es.get_name().to_string();
                let description = es.get_description().to_string();
                let id = es.get_id().to_string();
                if name.trim().is_empty() || es.get_busy() {
                    return;
                }
                // LOCAL playlist (id "local:<uuid>") — rename/description/
                // offline-only write the library.db repo; no Qobuz call.
                if local_playlist::is_local_id(&id) {
                    let offline_only = es.get_offline_only();
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let nm = name.trim().to_string();
                        let ds = description.trim().to_string();
                        let lid = id.clone();
                        let (nm2, ds2) = (nm.clone(), ds.clone());
                        let ok = tokio::task::spawn_blocking(move || {
                            let desc = if ds2.is_empty() { None } else { Some(ds2.as_str()) };
                            local_playlist::update_blocking(&lid, &nm2, desc, offline_only)
                        })
                        .await
                        .unwrap_or(false);
                        if !ok {
                            log::error!("[qbz-slint] update local playlist failed");
                            return;
                        }
                        let r2 = runtime.clone();
                        let w2 = weak.clone();
                        let h2 = handle.clone();
                        let rid = id.clone();
                        let rnm = nm.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            // Optimistic sidebar patch FIRST (the reload
                            // alone can show the pre-rename name — see
                            // sidebar::rename_entry), then reconcile.
                            sidebar::rename_entry(&w, &id, &nm);
                            let ps = w.global::<PlaylistState>();
                            // Only refresh the open detail header if this IS
                            // the open playlist.
                            if ps.get_id().as_str() == id {
                                ps.set_name(nm.into());
                                ps.set_description(ds.into());
                                ps.set_offline_only(offline_only);
                            }
                            w.global::<EditPlaylistState>().set_open(false);
                        });
                        // Hold the new name until the data source agrees
                        // (first pass for local: the DB read is already fresh).
                        reconcile_sidebar_after_rename(r2, w2, &h2, rid, rnm);
                    });
                    return;
                }
                let (Ok(pid),) = (id.parse::<u64>(),) else { return; };
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    let desc_opt = Some(description.trim());
                    match runtime
                        .core()
                        .update_playlist(pid, Some(name.trim()), desc_opt, None)
                        .await
                    {
                        Ok(_) => {
                            let nm = name.trim().to_string();
                            let ds = description.trim().to_string();
                            let r2 = runtime.clone();
                            let w2 = weak.clone();
                            let h2 = handle.clone();
                            let rid = id.clone();
                            let rnm = nm.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                // Optimistic sidebar patch FIRST — Qobuz's
                                // playlist/list can lag read-after-write, so
                                // the reload alone may show the old name (see
                                // sidebar::rename_entry).
                                sidebar::rename_entry(&w, &id, &nm);
                                w.global::<PlaylistState>().set_name(nm.into());
                                w.global::<PlaylistState>().set_description(ds.into());
                                w.global::<EditPlaylistState>().set_open(false);
                            });
                            // Hold the optimistic name until Qobuz's list
                            // agrees (bounded retries); replaces the plain
                            // reload that overwrote it with the stale name.
                            reconcile_sidebar_after_rename(r2, w2, &h2, rid, rnm);
                        }
                        Err(e) => log::error!("[qbz-slint] update playlist failed: {e}"),
                    }
                });
            });
    }
    {
        // Delete the playlist, then navigate back + refresh the sidebar.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<EditPlaylistActions>()
            .on_delete(move || {
                let Some(w) = weak.upgrade() else { return; };
                let id = w.global::<EditPlaylistState>().get_id().to_string();
                log::info!(
                    "[playlist-delete] requested: id='{id}' is_local={}",
                    local_playlist::is_local_id(&id)
                );
                // LOCAL playlist — delete the library.db entity (cascades
                // its membership rows), then back + sidebar reload.
                if local_playlist::is_local_id(&id) {
                    w.global::<EditPlaylistState>().set_busy(true);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    handle.clone().spawn(async move {
                        let lid = id.clone();
                        let nav_id = id.clone();
                        let ok = tokio::task::spawn_blocking(move || {
                            local_playlist::delete_blocking(&lid)
                        })
                        .await
                        .unwrap_or(false);
                        log::info!("[playlist-delete] local delete result -> {ok}");
                        let r2 = runtime.clone();
                        let w2 = weak.clone();
                        let h2 = handle.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            w.global::<EditPlaylistState>().set_busy(false);
                            if ok {
                                w.global::<EditPlaylistState>().set_open(false);
                                load_sidebar_playlists(r2, w2, &h2);
                                // §3: only step back when viewing THIS playlist's
                                // detail; otherwise stay on the invoking surface
                                // (the sidebar refresh above drops the row).
                                let on_detail = w.global::<NavState>().get_view() == ContentView::Playlist
                                    && w.global::<PlaylistState>().get_id().to_string() == nav_id;
                                if on_detail {
                                    w.global::<NavState>().invoke_request_back();
                                }
                            }
                        });
                    });
                    return;
                }
                let Ok(pid) = id.parse::<u64>() else {
                    log::warn!("[playlist-delete] non-numeric Qobuz id '{id}' — aborting");
                    return;
                };
                w.global::<EditPlaylistState>().set_busy(true);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let id_for_nav = id.clone();
                handle.clone().spawn(async move {
                    // Re-derive ownership server-side — the modal opens from
                    // surfaces (sidebar context menu / manager) that don't carry
                    // the owner flag, so never trust the UI here. OWNED => delete;
                    // FOLLOWED/subscribed (not owned) => unsubscribe. Qobuz's
                    // playlist/delete returns 200 but NO-OPS on a playlist you
                    // don't own (the "deleted ok but it stays" bug), so a followed
                    // playlist MUST go through unsubscribe.
                    let me = crate::library_db::current_user_id();
                    let owned = match runtime.core().get_playlist(pid).await {
                        Ok(p) => me.is_some_and(|uid| uid == p.owner.id),
                        Err(e) => {
                            log::warn!(
                                "[playlist-delete] {pid} owner check failed ({e}); treating as not-owned"
                            );
                            false
                        }
                    };
                    let res = if owned {
                        log::info!("[playlist-delete] {pid} OWNED -> delete");
                        runtime.core().delete_playlist(pid).await
                    } else {
                        log::info!("[playlist-delete] {pid} FOLLOWED -> unsubscribe");
                        runtime.core().unsubscribe_playlist(pid).await
                    };
                    match res {
                        Ok(()) => {
                            log::info!("[playlist-delete] {pid} removed ok (owned={owned})");
                            let r2 = runtime.clone();
                            let w2 = weak.clone();
                            let h2 = handle.clone();
                            let nav_id = id_for_nav.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                w.global::<EditPlaylistState>().set_busy(false);
                                w.global::<EditPlaylistState>().set_open(false);
                                load_sidebar_playlists(r2, w2, &h2);
                                // §3: only step back when viewing THIS playlist's
                                // detail; else stay on the invoking surface (the
                                // sidebar refresh above drops the row).
                                let on_detail = w.global::<NavState>().get_view() == ContentView::Playlist
                                    && w.global::<PlaylistState>().get_id().to_string() == nav_id;
                                if on_detail {
                                    w.global::<NavState>().invoke_request_back();
                                }
                            });
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] remove playlist failed: {e}");
                            let _ = weak.upgrade_in_event_loop(|w| {
                                w.global::<EditPlaylistState>().set_busy(false);
                            });
                        }
                    }
                });
            });
    }

    // Sidebar playlists — open a row, or create a new playlist.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SidebarActions>()
            .on_open_playlist(move |id| {
                nav::record(nav::NavEntry::Playlist(id.to_string()));
                navigate_playlist(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    id.to_string(),
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        // Populate the collapsed-sidebar folder flyout's playlist list.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_load_folder_popup(move |folder_id| {
                if let Some(w) = weak.upgrade() {
                    sidebar::load_folder_popup(&w, folder_id.as_str());
                }
            });
    }
    {
        // The "+" shortcut now opens the unified picker (PlaylistAddModal)
        // pre-deployed to its inline create row, instead of the retired
        // CreatePlaylistModal (spec §2/§3): same `open_for_ids` entry point
        // as every other "Add to playlist" trigger, just with an empty
        // id list (create-only, no pending tracks — `on_create_and_add`
        // already handles 0 carried ids as a no-op add, matching the old
        // create-without-adding shortcut).
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_create_playlist(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_picker::open_for_ids(&w, runtime.clone(), &handle, Vec::new(), false);
                    let picker = w.global::<PlaylistPickerState>();
                    picker.set_creating_open(true);
                    picker.set_create_name("".into());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<CreateFolderActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<CreateFolderState>().set_open(false);
                }
            });
    }
    {
        // Create a folder, then refresh the sidebar.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<CreateFolderActions>()
            .on_submit(move || {
                let Some(w) = weak.upgrade() else { return; };
                let name = w.global::<CreateFolderState>().get_name().to_string();
                if name.trim().is_empty() || w.global::<CreateFolderState>().get_creating() {
                    return;
                }
                w.global::<CreateFolderState>().set_creating(true);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    let nm = name.trim().to_string();
                    tokio::task::spawn_blocking(move || {
                        folders::create_folder(&nm);
                    })
                    .await
                    .ok();
                    let r2 = runtime.clone();
                    let w2 = weak.clone();
                    let h2 = handle.clone();
                    let _ = weak.upgrade_in_event_loop(move |w| {
                        w.global::<CreateFolderState>().set_creating(false);
                        w.global::<CreateFolderState>().set_open(false);
                        load_sidebar_playlists(r2, w2, &h2);
                    });
                });
            });
    }
    {
        // Toggle a folder's expanded state (cheap, rebuilds from cache).
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_toggle_folder(move |id| {
                if let Some(w) = weak.upgrade() {
                    sidebar::toggle_folder(&w, id.as_str());
                    refresh_sidebar_covers(&w);
                }
            });
    }
    {
        // Open the create-folder modal.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_create_folder(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<CreateFolderState>().set_name("".into());
                    w.global::<CreateFolderState>().set_creating(false);
                    w.global::<CreateFolderState>().set_open(true);
                }
            });
    }
    {
        // Delete a folder (its playlists fall back to root via the
        // library DB's ON DELETE SET NULL), then reload the sidebar.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_delete_folder(move |id| {
                let id = id.to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    let fid = id.clone();
                    tokio::task::spawn_blocking(move || folders::delete_folder(&fid))
                        .await
                        .ok();
                    load_sidebar_playlists(runtime, weak, &handle);
                });
            });
    }
    {
        // Move a playlist into a folder ("" = root). Optimistic local
        // rebuild + a DB write.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_move_playlist(move |playlist_id, folder_id| {
                let Some(w) = weak.upgrade() else { return; };
                let fid = folder_id.to_string();
                // LOCAL playlists (`local:<uuid>`) persist into the
                // local_playlists.folder_id column; Qobuz ones into
                // playlist_settings. Both join the SAME shared folders.
                if local_playlist::is_local_id(&playlist_id) {
                    let id = playlist_id.to_string();
                    sidebar::move_local_playlist_local(&w, &id, &fid);
                    refresh_sidebar_covers(&w);
                    handle.spawn(async move {
                        tokio::task::spawn_blocking(move || {
                            let opt = if fid.is_empty() { None } else { Some(fid.as_str()) };
                            folders::move_local_playlist(&id, opt);
                        })
                        .await
                        .ok();
                    });
                    return;
                }
                let Ok(pid) = playlist_id.parse::<u64>() else { return; };
                sidebar::move_playlist_local(&w, pid, &fid);
                refresh_sidebar_covers(&w);
                handle.spawn(async move {
                    tokio::task::spawn_blocking(move || {
                        let opt = if fid.is_empty() { None } else { Some(fid.as_str()) };
                        folders::move_playlist(pid, opt);
                    })
                    .await
                    .ok();
                });
            });
    }
    {
        // Pick a playlist sort option (name/recent/tracks/playcount/custom).
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_set_sort(move |option| {
                if let Some(w) = weak.upgrade() {
                    sidebar::set_sort(&w, option.as_str());
                    refresh_sidebar_covers(&w);
                }
            });
    }
    {
        // Re-run the playlist-name filter as the search input edits.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_search_changed(move |query| {
                if let Some(w) = weak.upgrade() {
                    sidebar::set_search(&w, query.as_str());
                    refresh_sidebar_covers(&w);
                }
            });
    }
    {
        // Refresh — re-fetch the playlist list from the network.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_refresh_playlists(move || {
                load_sidebar_playlists(runtime.clone(), weak.clone(), &handle);
            });
    }
    {
        // Manage playlists — open the full Playlist Manager view.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<SidebarActions>()
            .on_manage_playlists(move || {
                nav::record(nav::NavEntry::PlaylistManager);
                playlist_manager::navigate(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        // Import playlist — open the importer modal fully reset, with the
        // folder dropdown rebuilt from the current sidebar folder list.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_import_playlist(move || {
                if let Some(w) = weak.upgrade() {
                    playlist_import::open(&w);
                }
            });
    }
    {
        // Edit playlist (sidebar context menu) — open the edit-playlist
        // modal, prefilled from the cached name + description.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_edit_playlist(move |id| {
                let Some(w) = weak.upgrade() else { return };
                let es = w.global::<EditPlaylistState>();
                // LOCAL playlist row — prefill from the sidebar's local
                // cache (name/description/offline-only).
                if local_playlist::is_local_id(id.as_str()) {
                    let (name, description, offline_only) =
                        sidebar::local_playlist_meta(id.as_str())
                            .unwrap_or_else(|| (id.to_string(), String::new(), false));
                    es.set_id(id);
                    es.set_name(name.into());
                    es.set_description(description.into());
                    es.set_is_local(true);
                    es.set_offline_only(offline_only);
                    es.set_busy(false);
                    es.set_open(true);
                    return;
                }
                let (name, description) = id
                    .parse::<u64>()
                    .ok()
                    .and_then(sidebar::playlist_name_desc)
                    .unwrap_or_else(|| (id.to_string(), String::new()));
                es.set_id(id);
                es.set_name(name.into());
                es.set_description(description.into());
                es.set_is_local(false);
                es.set_offline_only(false);
                es.set_busy(false);
                es.set_open(true);
            });
    }
    {
        // Add to Mixtape/Collection (sidebar playlist context menu) — build a
        // 1-item playlist payload from the cached SidebarEntry row + the cached
        // track count, then open the global AddToMixtapeModal. Because the
        // item_type is "playlist", `open_add_to_mixtape` computes restrict=true
        // → the picker lists mixtapes only and hides the "+ Collections" chip (a
        // playlist can't live in a Collection). Mirrors the PlaylistManager path.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_add_to_mixtape(move |id| {
                use slint::Model;
                let Some(w) = weak.upgrade() else { return };
                let model = w.global::<SidebarState>().get_entries();
                let Some(row) = (0..model.row_count())
                    .filter_map(|i| model.row_data(i))
                    .find(|e| e.kind == "playlist" && e.id == id)
                else {
                    return;
                };
                let artwork = row.url1.to_string();
                let item = myqbz_add::AddItem {
                    item_type: "playlist".into(),
                    source: "qobuz".into(),
                    source_item_id: id.to_string(),
                    title: row.name.to_string(),
                    subtitle: None,
                    artwork_url: (!artwork.is_empty()).then_some(artwork),
                    year: None,
                    // SidebarEntry doesn't carry the count; pull it from the
                    // sidebar cache by id (None if unknown — it's optional).
                    track_count: id
                        .parse::<u64>()
                        .ok()
                        .and_then(sidebar::playlist_track_count)
                        .map(|n| n as i32),
                };
                open_add_to_mixtape(weak.clone(), handle.clone(), vec![item]);
            });
    }
    {
        // Edit folder (sidebar context menu) — open the folder editor.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_edit_folder(move |id| {
                let Some(w) = weak.upgrade() else { return };
                open_folder_editor(&w, id);
            });
    }
    {
        // Filter the move-to-folder menu list by a substring query.
        let weak = window.as_weak();
        window
            .global::<SidebarActions>()
            .on_search_folders(move |query| {
                if let Some(w) = weak.upgrade() {
                    sidebar::search_menu_folders(&w, query.as_str());
                }
            });
    }
    {
        // Hide playlist from the sidebar (local setting), then reload.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_hide_playlist(move |id| {
                let Ok(pid) = id.parse::<u64>() else { return };
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    tokio::task::spawn_blocking(move || folders::set_hidden(pid, true))
                        .await
                        .ok();
                    load_sidebar_playlists(runtime, weak, &handle);
                });
            });
    }
    {
        // Hide folder from the sidebar (local setting), then reload.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<SidebarActions>()
            .on_hide_folder(move |id| {
                let fid = id.to_string();
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                handle.clone().spawn(async move {
                    tokio::task::spawn_blocking(move || folders::set_folder_hidden(&fid, true))
                        .await
                        .ok();
                    load_sidebar_playlists(runtime, weak, &handle);
                });
            });
    }

    // === Playlist Manager actions ======================================
    wire_playlist_manager(&window, &app_runtime, &tokio_rt, &image_cache);
    wire_myqbz(&window, &app_runtime, &tokio_rt, &image_cache);
    wire_myqbz_detail(&window, &app_runtime, &tokio_rt, &image_cache);
    {
        let weak = window.as_weak();
        window
            .global::<CreatePlaylistActions>()
            .on_close(move || {
                if let Some(w) = weak.upgrade() {
                    w.global::<CreatePlaylistState>().set_open(false);
                }
            });
    }
    {
        // Create the playlist, then refresh the sidebar + open it.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<CreatePlaylistActions>()
            .on_submit(move || {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                use slint::Model;
                let state = w.global::<CreatePlaylistState>();
                let name = state.get_name().to_string();
                let description = state.get_description().to_string();
                let is_public = state.get_is_public();
                // Resolve the selected folder id ("" = No folder).
                let folder_idx = state.get_folder_index();
                let folder_id = state
                    .get_folder_ids()
                    .row_data(folder_idx.max(0) as usize)
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                if name.trim().is_empty() || state.get_creating() {
                    return;
                }
                // D8: offline-only toggle ON — or the app is offline (always
                // local then) — creates a LOCAL playlist in library.db. The
                // online + toggle OFF path below stays byte-unchanged.
                let offline_now = offline_mode::engine().is_offline();
                if state.get_offline_only() || offline_now {
                    // Offline-only when the user opted in; a playlist forced
                    // local by being offline keeps the flag too (it can be
                    // unmarked later in Edit to enable "Upload to Qobuz").
                    state.set_creating(true);
                    let runtime = runtime.clone();
                    let weak = weak.clone();
                    let handle = handle.clone();
                    let image_cache = image_cache.clone();
                    handle.clone().spawn(async move {
                        let nm = name.trim().to_string();
                        let ds = description.trim().to_string();
                        let created = tokio::task::spawn_blocking(move || {
                            let desc = if ds.is_empty() { None } else { Some(ds.as_str()) };
                            local_playlist::create_blocking(&nm, desc, true)
                        })
                        .await
                        .ok()
                        .flatten();
                        let weak2 = weak.clone();
                        let r2 = runtime.clone();
                        let h2 = handle.clone();
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            w.global::<CreatePlaylistState>().set_creating(false);
                            match created {
                                Some(new_id) => {
                                    w.global::<CreatePlaylistState>().set_open(false);
                                    load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                    nav::record(nav::NavEntry::Playlist(new_id.clone()));
                                    navigate_playlist(r2, weak2.clone(), &h2, image_cache, new_id);
                                    update_nav_flags(&w);
                                }
                                None => {
                                    log::error!("[qbz-slint] create local playlist failed");
                                }
                            }
                        });
                    });
                    return;
                }
                state.set_creating(true);
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let image_cache = image_cache.clone();
                handle.clone().spawn(async move {
                    let desc = description.trim();
                    let desc_opt = if desc.is_empty() { None } else { Some(desc) };
                    match runtime.core().create_playlist(name.trim(), desc_opt, is_public).await {
                        Ok(playlist) => {
                            let new_id = playlist.id.to_string();
                            // Assign to the chosen folder (local DB) before
                            // the sidebar reloads, mirroring Tauri.
                            if !folder_id.is_empty() {
                                let pid = playlist.id;
                                let fid = folder_id.clone();
                                tokio::task::spawn_blocking(move || {
                                    folders::move_playlist(pid, Some(fid.as_str()));
                                })
                                .await
                                .ok();
                            }
                            let weak2 = weak.clone();
                            let r2 = runtime.clone();
                            let h2 = handle.clone();
                            let ic2 = image_cache.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                w.global::<CreatePlaylistState>().set_creating(false);
                                w.global::<CreatePlaylistState>().set_open(false);
                                load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                nav::record(nav::NavEntry::Playlist(new_id.clone()));
                                navigate_playlist(r2, weak2.clone(), &h2, ic2, new_id);
                                update_nav_flags(&w);
                            });
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] create playlist failed: {e}");
                            let _ = weak.upgrade_in_event_loop(|w| {
                                w.global::<CreatePlaylistState>().set_creating(false);
                            });
                        }
                    }
                });
            });
    }

    // ---- HiFi Wizard (DAC setup) — Slice 6 (check step) ----
    {
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_open(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            dac_wizard::open_immediate(&w);
            // Probe the audio stack off the UI thread; fill the check step when done.
            let weak2 = w.as_weak();
            handle.spawn_blocking(move || {
                let health = qbz_audio::audio_stack_health();
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    dac_wizard::apply_health(&w, health);
                });
            });
        });
    }
    {
        let weak = window.as_weak();
        window
            .global::<DacWizardActions>()
            .on_set_distro(move |index| {
                if let Some(w) = weak.upgrade() {
                    dac_wizard::set_distro(&w, index);
                }
            });
    }
    {
        let weak = window.as_weak();
        window.global::<DacWizardActions>().on_set_init(move |index| {
            if let Some(w) = weak.upgrade() {
                dac_wizard::set_init(&w, index);
            }
        });
    }
    {
        // Enumerate DACs (Slice 7) off the UI thread when entering the step.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_run_detect(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            dac_wizard::begin_detect(&w);
            let weak2 = w.as_weak();
            handle.spawn_blocking(move || {
                let data = dac_wizard::detect_blocking();
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    dac_wizard::apply_candidates(&w, data);
                });
            });
        });
    }
    {
        let weak = window.as_weak();
        window.global::<DacWizardActions>().on_toggle_dac(move |i| {
            if let Some(w) = weak.upgrade() {
                dac_wizard::toggle_dac(&w, i);
            }
        });
    }
    {
        let weak = window.as_weak();
        window
            .global::<DacWizardActions>()
            .on_validate_manual(move |t| {
                if let Some(w) = weak.upgrade() {
                    dac_wizard::validate_manual(&w, t.as_str());
                }
            });
    }
    {
        // Self-service playback test (Slice 9): resolve the 4 curated tracks
        // (id-hint then "artist title" search), route output to the DAC under
        // test, and play them. The N6 read-back is driven by on_poll_test.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_start_test(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            dac_wizard::begin_test(&w);
            let runtime = runtime.clone();
            let weak2 = w.as_weak();
            let play_handle = handle.clone();
            handle.spawn(async move {
                let mut tracks: Vec<qbz_models::Track> = Vec::new();
                for seed in dac_wizard::TEST_SEEDS.iter() {
                    let mut chosen = match runtime.core().get_track(seed.id_hint).await {
                        Ok(t) if dac_wizard::track_matches_seed(&t, seed) => Some(t),
                        _ => None,
                    };
                    if chosen.is_none() {
                        let q = format!("{} {}", seed.artist, seed.title);
                        if let Ok(page) = runtime.core().search_tracks(&q, 10, 0, None).await {
                            chosen = page
                                .items
                                .into_iter()
                                .find(|t| dac_wizard::track_matches_seed(t, seed));
                        }
                    }
                    if let Some(t) = chosen {
                        tracks.push(t);
                    }
                }
                // Keep the resolved tracks so the user can jump between them.
                dac_wizard::stash_test_tracks(tracks.clone());
                let runtime2 = runtime.clone();
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    if tracks.is_empty() {
                        w.global::<DacWizardState>()
                            .set_test_requested_label("Couldn't load the test tracks (offline?)".into());
                        return;
                    }
                    crate::playback::play_tracks(runtime2, w.as_weak(), play_handle, tracks, 0);
                });
            });
        });
    }
    {
        // Poll the requested vs negotiated rate while the test plays.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_poll_test(move || {
            if weak.upgrade().is_none() {
                return;
            }
            let runtime = runtime.clone();
            let weak2 = weak.clone();
            handle.spawn_blocking(move || {
                let player = runtime.core().player();
                let req_rate = player.state.get_sample_rate();
                let req_bits = player.state.get_bit_depth();
                let negotiated = qbz_audio::negotiated_active_rate();
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    dac_wizard::apply_poll(&w, req_rate, req_bits, negotiated);
                });
            });
        });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        window.global::<DacWizardActions>().on_stop_test(move || {
            let _ = runtime.core().pause();
            if let Some(w) = weak.upgrade() {
                dac_wizard::end_test(&w);
            }
        });
    }
    {
        // Jump straight to one of the test tracks (skip the long waits) by
        // re-setting the queue at that index via the working play path.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<DacWizardActions>()
            .on_test_play_index(move |i| {
                let tracks = dac_wizard::test_tracks();
                if tracks.is_empty() {
                    return;
                }
                let start = (i.max(0) as usize).min(tracks.len().saturating_sub(1));
                crate::playback::play_tracks(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    tracks,
                    start,
                );
            });
    }
    {
        // "Use my own music": start the read-back without queuing the test
        // tracks — the user plays whatever they want; the poll reads the rate.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_verify_own(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            let runtime = runtime.clone();
            let weak2 = w.as_weak();
            handle.spawn(async move {
                // Guardrail: don't start a read-back on an empty queue.
                let (tracks, _) = runtime.core().get_all_queue_tracks().await;
                let empty = tracks.is_empty();
                if !empty {
                    let _ = runtime.core().resume();
                }
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    if empty {
                        dac_wizard::queue_empty_notice(&w);
                    } else {
                        dac_wizard::begin_test(&w);
                    }
                });
            });
        });
    }
    {
        // Generate the per-DAC copy-paste config (Slice 10): re-probe rates off
        // the UI thread, then fill the review step.
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<DacWizardActions>().on_gen_configs(move || {
            let Some(w) = weak.upgrade() else {
                return;
            };
            let dacs = dac_wizard::checked_dacs(&w);
            let weak2 = w.as_weak();
            handle.spawn_blocking(move || {
                let data = dac_wizard::gen_configs_blocking(dacs);
                let _ = weak2.upgrade_in_event_loop(move |w| {
                    dac_wizard::apply_configs(&w, data);
                });
            });
        });
    }
    {
        let weak = window.as_weak();
        window.global::<DacWizardActions>().on_toggle_config(move |i| {
            if let Some(w) = weak.upgrade() {
                dac_wizard::toggle_config(&w, i);
            }
        });
    }
    {
        window
            .global::<DacWizardActions>()
            .on_copy_command(move |cmd| {
                share::copy_to_clipboard(cmd.to_string());
            });
    }

    // ---- Sandbox (Flatpak/Snap) settings section ----
    // Seed the install method once (drives section visibility) and wire the
    // copy-to-clipboard action for the permission commands.
    {
        let method = qbz_app::diagnostics::system_info().install_method;
        window.global::<SandboxState>().set_install_method(method.into());
        window
            .global::<SandboxState>()
            .on_copy_command(move |cmd| {
                share::copy_to_clipboard(cmd.to_string());
            });
    }

    // ---- Playlist Importer (public playlists) — spec §3.3 ----
    {
        // No cancel exists: a running import task continues to completion
        // (§1.8); closing only hides the modal.
        let weak = window.as_weak();
        window.global::<PlaylistImportActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                w.global::<PlaylistImportState>().set_open(false);
            }
        });
    }
    {
        // Provider detection per keystroke (Slint 1.16 has no `.contains`).
        let weak = window.as_weak();
        window
            .global::<PlaylistImportActions>()
            .on_url_edited(move |text| {
                if let Some(w) = weak.upgrade() {
                    playlist_import::on_url_edited(&w, text.as_str());
                }
            });
    }
    {
        window
            .global::<PlaylistImportActions>()
            .on_name_edited(move |text| {
                playlist_import::on_name_edited(text.as_str());
            });
    }
    {
        // Step A: fetch the preview (no session needed).
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window.global::<PlaylistImportActions>().on_fetch(move || {
            let Some(w) = weak.upgrade() else { return; };
            let Some(url) = playlist_import::begin_fetch(&w) else {
                return;
            };
            // A reopen mid-fetch bumps the generation; the stale preview
            // result must not land on the fresh modal (§1.8).
            let generation = playlist_import::current_generation();
            let weak = weak.clone();
            handle.spawn(async move {
                let res = qbz_playlist_import::preview_public_playlist(&url).await;
                let _ = weak.upgrade_in_event_loop(move |w| {
                    if generation != playlist_import::current_generation() {
                        return;
                    }
                    match res {
                        Ok(p) => playlist_import::apply_preview_ok(&w, &url, p),
                        Err(e) => playlist_import::apply_preview_err(&w, &e.to_string()),
                    }
                });
            });
        });
    }
    {
        // Step B: execute the import (re-fetches the URL internally —
        // Tauri behavior, kept) with live sink progress.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<PlaylistImportActions>()
            .on_execute(move || {
                let Some(w) = weak.upgrade() else { return; };
                let Some(args) = playlist_import::begin_execute(&w) else {
                    return;
                };
                let runtime = runtime.clone();
                let weak = weak.clone();
                let handle = handle.clone();
                let image_cache = image_cache.clone();
                handle.clone().spawn(async move {
                    // Tauri's RequiresUserSession gate: execute needs a
                    // logged-in client (the preview does not).
                    let client = runtime.core().client().read().await.clone();
                    let Some(client) = client else {
                        let g = args.generation;
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            if g == playlist_import::current_generation() {
                                playlist_import::apply_execute_err(
                                    &w,
                                    "Not logged in to Qobuz",
                                );
                            }
                            toast::show(&w, "Playlist import failed", ToastKind::Error);
                        });
                        return;
                    };
                    let sink: Arc<dyn qbz_playlist_import::ImportProgressSink> = Arc::new(
                        playlist_import::SlintSink::new(weak.clone(), args.generation),
                    );
                    let res = qbz_playlist_import::import_public_playlist(
                        &args.url,
                        &client,
                        args.name_override.as_deref(),
                        false, // is_public — Tauri hardcodes false, no toggle
                        sink,
                    )
                    .await;
                    match res {
                        Ok(summary) => {
                            // reco: NOT logged. The importer is a bulk external
                            // import (Spotify/Apple/...), not a per-track taste
                            // action, and Tauri never logged it — left unlogged
                            // for parity. (Re-evaluate if the owner wants it.)
                            // Assign every created part to the chosen folder
                            // (local DB) BEFORE the sidebar reload — mirrors
                            // the create-playlist precedent above.
                            if !args.folder_id.is_empty() {
                                for pid in &summary.qobuz_playlist_ids {
                                    let (pid, fid) = (*pid, args.folder_id.clone());
                                    tokio::task::spawn_blocking(move || {
                                        folders::move_playlist(pid, Some(fid.as_str()));
                                    })
                                    .await
                                    .ok();
                                }
                            }
                            let g = args.generation;
                            let weak2 = weak.clone();
                            let r2 = runtime.clone();
                            let h2 = handle.clone();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                // Toast + sidebar refresh fire even after a
                                // close mid-import (§1.8); the generation
                                // guard keeps a stale run's writes off a
                                // reopened modal's fresh state.
                                if g == playlist_import::current_generation() {
                                    playlist_import::apply_execute_ok(&w, &summary);
                                }
                                if summary.matched_tracks > 0 {
                                    toast::show(&w, "Playlist imported", ToastKind::Success);
                                }
                                load_sidebar_playlists(r2.clone(), weak2.clone(), &h2);
                                if let Some(first) = summary.qobuz_playlist_ids.first() {
                                    // Navigate only while the modal is still
                                    // open AND this run is current (§1.8).
                                    if g == playlist_import::current_generation()
                                        && w.global::<PlaylistImportState>().get_open()
                                    {
                                        nav::record(nav::NavEntry::Playlist(first.to_string()));
                                        navigate_playlist(
                                            r2,
                                            weak2,
                                            &h2,
                                            image_cache,
                                            first.to_string(),
                                        );
                                        update_nav_flags(&w);
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            let g = args.generation;
                            let msg = e.to_string();
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                if g == playlist_import::current_generation() {
                                    playlist_import::apply_execute_err(&w, &msg);
                                }
                                toast::show(&w, "Playlist import failed", ToastKind::Error);
                            });
                        }
                    }
                });
            });
    }

    // handler.
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_select_tab(move |id| {
                if id.as_str() == "all" {
                    nav::record(nav::NavEntry::Favorites {
                        tab: "all".to_string(),
                    });
                    if let Some(w) = weak.upgrade() {
                        update_nav_flags(&w);
                    }
                    navigate_library_all(
                        runtime.clone(),
                        weak.clone(),
                        &handle,
                        image_cache.clone(),
                    );
                    return;
                }
                let Some(tab) = favorites::FavTab::from_tab_id(id.as_str()) else {
                    // Playlists / Labels: just switch the visible tab,
                    // their content is not implemented yet.
                    if let Some(w) = weak.upgrade() {
                        w.global::<FavoritesState>().set_active_tab(id);
                    }
                    return;
                };
                // Each favorites tab is its own history page (mirrors the
                // Discover tabs) so back/forward moves between them.
                nav::record(nav::NavEntry::Favorites { tab: id.to_string() });
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
                navigate_favorites(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    tab,
                    id.as_str(),
                );
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_open_album(move |id| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_open_album(id);
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_open_artist(move |id| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_open_artist(id);
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_open_label(move |id, name| {
                let Ok(label_id) = id.parse::<u64>() else {
                    return;
                };
                let name = name.to_string();
                nav::record(nav::NavEntry::Label {
                    id: label_id,
                    name: name.clone(),
                });
                navigate_label(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    label_id,
                    name,
                );
            });
    }
    {
        // Favorite playlist click — open the playlist detail view.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_open_playlist(move |id| {
                nav::record(nav::NavEntry::Playlist(id.to_string()));
                navigate_playlist(
                    runtime.clone(),
                    weak.clone(),
                    &handle,
                    image_cache.clone(),
                    id.to_string(),
                );
                if let Some(w) = weak.upgrade() {
                    update_nav_flags(&w);
                }
            });
    }
    {
        // Switch the Playlists sub-tab (Library / Following) + re-derive.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_playlists_set_sub_tab(move |sub| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_playlists_sub_tab(sub);
                    favorites::derive_playlists(&w);
                }
            });
    }
    {
        // Local search over the loaded favorite playlists (name | owner).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_search_playlists(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_playlists_search(q);
                    favorites::derive_playlists(&w);
                }
            });
    }
    {
        // Playlists grid/list view toggle (persisted).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_playlists_set_view(move |v| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_playlists_view_mode(v);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Local search over the loaded favorite artists (name).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_search_artists(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_artists_search(q);
                    favorites::derive_artists(&w);
                }
            });
    }
    {
        // Artists header Shuffle = open a random visible artist (random
        // ARTIST, not a random album — matches Tauri).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_artists_shuffle(move || {
                if let Some(w) = weak.upgrade() {
                    if let Some(id) = favorites::random_visible_artist(&w) {
                        w.invoke_open_artist(id.into());
                    }
                }
            });
    }
    {
        // Playlists "random" — play a random visible playlist (reuses the
        // playlist-action "play" path).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_playlists_random(move || {
                if let Some(w) = weak.upgrade() {
                    if let Some(id) = favorites::random_visible_playlist(&w) {
                        w.global::<FavoritesActions>()
                            .invoke_playlist_action(id.into(), "play".into());
                    }
                }
            });
    }
    {
        // Labels "random" — open a random visible label's landing.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_labels_random(move || {
                if let Some(w) = weak.upgrade() {
                    if let Some((id, name)) = favorites::random_visible_label(&w) {
                        w.global::<FavoritesActions>()
                            .invoke_open_label(id.into(), name.into());
                    }
                }
            });
    }
    {
        // Group the favorite artists (off / A-Z) — persisted.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_artists_set_group(move |g| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>()
                        .set_artists_group_enabled(g == "alpha");
                    favorites::derive_artists(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Artists grid <-> sidepanel view toggle (persisted). Switching back to
        // grid clears the sidepanel selection (matches Tauri).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_artists_set_view(move |v| {
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<FavoritesState>();
                    st.set_artists_view_mode(v.clone());
                    if v == "grid" {
                        st.set_selected_artist_id("".into());
                    }
                    // Rebuild grouped/alpha for the new mode (the sidepanel
                    // left list is always grouped).
                    favorites::derive_artists(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Sidepanel: load + show the selected artist's albums, reusing the
        // standalone artist page's /artist/page release_type classifier.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_select_artist(move |id, name| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                let st = w.global::<FavoritesState>();
                st.set_selected_artist_id(id.clone());
                st.set_selected_artist_name(name);
                st.set_selected_albums_loading(true);
                st.set_selected_albums_error("".into());
                let runtime = runtime.clone();
                let weak2 = weak.clone();
                let image_cache = image_cache.clone();
                let id_s = id.to_string();
                handle.spawn(async move {
                    match artist::load_artist(&runtime, &id_s).await {
                        Ok(data) => {
                            let sections = data.release_sections;
                            let jobs = favorites::selected_artist_artwork_jobs(&sections);
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                favorites::apply_selected_artist(&w, sections);
                            });
                            artwork::spawn_loads(jobs, weak2.clone(), image_cache.clone());
                        }
                        Err(e) => {
                            log::error!("[qbz-slint] sidepanel artist {id_s} load failed: {e}");
                            let _ = weak2.upgrade_in_event_loop(move |w| {
                                let st = w.global::<FavoritesState>();
                                st.set_selected_albums_loading(false);
                                st.set_selected_albums_error(e.into());
                            });
                        }
                    }
                });
            });
    }
    {
        // Playlist card actions: play / play-next / queue / share / favorite.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_playlist_action(move |id, action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "share" => share::copy_to_clipboard(share::qobuz_playlist_url(&id)),
                    "favorite" => {
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let in_playlists_tab = w
                            .global::<FavoritesState>()
                            .get_active_tab()
                            .to_string()
                            == "playlists";
                        if in_playlists_tab {
                            // Favorites › Playlists: Library sub-tab un-favorites
                            // in place (drop the row); Following sub-tab adds to
                            // the local Library (per user decision).
                            let library = w
                                .global::<FavoritesState>()
                                .get_playlists_sub_tab()
                                .to_string()
                                != "following";
                            let fav = !library;
                            handle.spawn_blocking(move || {
                                crate::library_db::with_db(|db| db.set_playlist_favorite(pid, fav));
                            });
                            if library {
                                favorites::remove_playlist_row(&w, &id);
                            }
                        } else {
                            // Library "All" (mixed feed): authoritative toggle by
                            // the DB state — a foreign card can't know it, and the
                            // owned-but-unhearted case must ADD, not remove.
                            playlist_toggle_favorite_by_id(handle.clone(), weak.clone(), pid, false);
                        }
                    }
                    "follow" => {
                        if let Ok(pid) = id.parse::<u64>() {
                            playlist_set_follow_by_id(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                pid,
                                true,
                                false,
                            );
                        }
                    }
                    "unfollow" => {
                        if let Ok(pid) = id.parse::<u64>() {
                            playlist_set_follow_by_id(
                                runtime.clone(),
                                weak.clone(),
                                handle.clone(),
                                pid,
                                false,
                                false,
                            );
                            // In the Favorites › Playlists Following sub-tab,
                            // unfollowing removes the row (mirrors un-favorite).
                            let fs = w.global::<FavoritesState>();
                            if fs.get_active_tab().to_string() == "playlists"
                                && fs.get_playlists_sub_tab().to_string() == "following"
                            {
                                favorites::remove_playlist_row(&w, &id);
                            }
                        }
                    }
                    "copy" => {
                        if let Ok(pid) = id.parse::<u64>() {
                            playlist_copy_by_id(runtime.clone(), weak.clone(), handle.clone(), pid, false);
                        }
                    }
                    act => {
                        // play / play-next / queue: fetch the playlist's tracks,
                        // then play or enqueue.
                        let Ok(pid) = id.parse::<u64>() else {
                            return;
                        };
                        let runtime = runtime.clone();
                        let weak2 = weak.clone();
                        let handle2 = handle.clone();
                        let act = act.to_string();
                        handle.spawn(async move {
                            let tracks = match runtime.core().get_playlist(pid).await {
                                Ok(p) => p.tracks.map(|t| t.items).unwrap_or_default(),
                                Err(e) => {
                                    log::error!("[qbz-slint] playlist {pid} load failed: {e}");
                                    return;
                                }
                            };
                            if tracks.is_empty() {
                                return;
                            }
                            match act.as_str() {
                                "play-next" => {
                                    playback::enqueue_tracks(runtime, handle2, tracks, true)
                                }
                                "queue" => {
                                    playback::enqueue_tracks(runtime, handle2, tracks, false)
                                }
                                _ => {
                                    playback::play_tracks(runtime, weak2, handle2, tracks, 0);
                                }
                            }
                        });
                    }
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_play_track(move |id| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_media_action("track".into(), id, "play".into());
                }
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_track_action(move |id, action| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_media_action("track".into(), id, action);
                }
            });
    }
    {
        // Favorite album card actions (play / queue / favorite / go-to)
        // route through the album media-action arms.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_album_action(move |id, action| {
                if let Some(w) = weak.upgrade() {
                    w.invoke_media_action("album".into(), id, action);
                }
            });
    }
    // ── Library "All" mixed feed — toolbar handlers ──
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<LibraryAllActions>().on_search(move |q| {
            if let Some(w) = weak.upgrade() {
                w.global::<LibraryAllState>().set_search(q);
                library_all::derive(&w);
                let jobs = library_all::artwork_jobs(&w);
                artwork::spawn_search_loads(jobs, weak.clone(), image_cache.clone());
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window.global::<LibraryAllActions>().on_set_sort(move |key| {
            if let Some(w) = weak.upgrade() {
                // Re-selecting the active field toggles asc/desc (PlaylistView
                // pattern); a new field resets to its natural direction.
                library_all::set_sort(&w, key.as_str());
                let jobs = library_all::artwork_jobs(&w);
                artwork::spawn_search_loads(jobs, weak.clone(), image_cache.clone());
            }
        });
    }
    {
        let weak = window.as_weak();
        window.global::<LibraryAllActions>().on_set_view(move |mode| {
            if let Some(w) = weak.upgrade() {
                w.global::<LibraryAllState>().set_view_mode(mode);
            }
        });
    }
    {
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<LibraryAllActions>()
            .on_toggle_source(move |which| {
                if let Some(w) = weak.upgrade() {
                    let st = w.global::<LibraryAllState>();
                    match which.as_str() {
                        "purchases" => st.set_show_purchases(!st.get_show_purchases()),
                        "favorites" => st.set_show_favorites(!st.get_show_favorites()),
                        "following" => st.set_show_following(!st.get_show_following()),
                        "local" => st.set_show_local(!st.get_show_local()),
                        _ => {}
                    }
                    library_all::derive(&w);
                    let jobs = library_all::artwork_jobs(&w);
                    artwork::spawn_search_loads(jobs, weak.clone(), image_cache.clone());
                }
            });
    }
    {
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window.global::<LibraryAllActions>().on_retry(move || {
            navigate_library_all(
                runtime.clone(),
                weak.clone(),
                &handle,
                image_cache.clone(),
            );
        });
    }
    {
        // Local search over the loaded favorite albums (title / artist).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_search(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_albums_search(q);
                    favorites::derive_albums(&w);
                }
            });
    }
    {
        // Sort the favorite albums (default / title / artist).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_set_sort(move |s| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_albums_sort_by(s);
                    favorites::derive_albums(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Albums grid/list view toggle (persisted).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_set_view(move |v| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_albums_view_mode(v);
                    // Switching to the (non-windowed) list view needs covers
                    // the grid's window may have evicted.
                    favorites::albums_view_mode_changed(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Windowed albums grid: dispatch covers for the reported row band
        // and evict the ones far outside it.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_window_changed(move |first, last| {
                if let Some(w) = weak.upgrade() {
                    favorites::albums_window_changed(&w, first, last);
                }
            });
    }
    {
        // Group the favorite albums (off / alpha / artist).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_set_group(move |g| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_albums_group_mode(g);
                    favorites::derive_albums(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Play a random album from the visible favorites set.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_albums_shuffle(move || {
                if let Some(w) = weak.upgrade() {
                    if let Some(id) = favorites::random_visible_album(&w) {
                        w.invoke_media_action("album".into(), id.into(), "play".into());
                    }
                }
            });
    }
    {
        // Un-favorite a track from the favorites list: fade the row, remove
        // the favorite on the server, then drop the row after the fade.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_unfavorite_track(move |id| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                // Offline = read-only hearts (spec 4.3).
                if offline_mode::engine().is_offline() {
                    toast::info(&w, "Not available offline");
                    return;
                }
                favorites::mark_track_removing(&w, &id);
                if let Ok(tid) = id.parse::<u64>() {
                    crate::fav_cache::set(tid, false);
                }
                let id_srv = id.to_string();
                let runtime = runtime.clone();
                handle.spawn(async move {
                    if let Err(e) = runtime.core().remove_favorite("track", &id_srv).await {
                        log::error!("[qbz-slint] unfavorite track {id_srv} failed: {e}");
                    }
                });
                let weak2 = weak.clone();
                let id_rm = id.to_string();
                slint::Timer::single_shot(std::time::Duration::from_millis(280), move || {
                    if let Some(w) = weak2.upgrade() {
                        favorites::remove_track_row(&w, &id_rm);
                    }
                });
            });
    }
    {
        // Un-favorite an album from the favorites list (same fade + remove).
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_unfavorite_album(move |id| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                favorites::mark_album_removing(&w, &id);
                // Keep the favorite-album cache in sync so the album-header
                // heart reflects an unfavorite done from the Favorites view.
                crate::fav_cache::set_album(&id, false);
                // Empty the heart on any other surface currently showing this
                // album (artist discography, carousels, search) — the
                // favorites rows themselves fade out and are removed below.
                set_album_row_favorite(&w, &id, false);
                let id_srv = id.to_string();
                let runtime = runtime.clone();
                handle.spawn(async move {
                    if let Err(e) = runtime.core().remove_favorite("album", &id_srv).await {
                        log::error!("[qbz-slint] unfavorite album {id_srv} failed: {e}");
                    }
                });
                let weak2 = weak.clone();
                let id_rm = id.to_string();
                slint::Timer::single_shot(std::time::Duration::from_millis(280), move || {
                    if let Some(w) = weak2.upgrade() {
                        favorites::remove_album_row(&w, &id_rm);
                    }
                });
            });
    }
    {
        // Retry loading the current favorites tab after a load error.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_retry_load(move || {
                if let Some(w) = weak.upgrade() {
                    let tab_id = w.global::<FavoritesState>().get_active_tab().to_string();
                    if let Some(tab) = favorites::FavTab::from_tab_id(&tab_id) {
                        navigate_favorites(
                            runtime.clone(),
                            weak.clone(),
                            &handle,
                            image_cache.clone(),
                            tab,
                            &tab_id,
                        );
                    }
                }
            });
    }
    {
        // Local search over the loaded favorite tracks (title / artist /
        // album), re-deriving the rendered list.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_search_tracks(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_tracks_search(q);
                    favorites::derive_tracks(&w);
                }
            });
    }
    {
        // Local search over the loaded favorite labels (name).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_search_labels(move |q| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_labels_search(q);
                    favorites::derive_labels(&w);
                }
            });
    }
    {
        // Group the favorite tracks (off / album / artist / name).
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_tracks_set_group(move |g| {
                if let Some(w) = weak.upgrade() {
                    w.global::<FavoritesState>().set_tracks_group_mode(g);
                    favorites::derive_tracks(&w);
                    favorites_prefs::save(&w);
                }
            });
    }
    {
        // Play all favorite tracks as a fresh queue.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_play_all_tracks(move || {
                playback::play_tracks(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    favorites::play_tracks(),
                    0,
                );
            });
    }
    {
        // Shuffle-play the favorite tracks.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        window
            .global::<FavoritesActions>()
            .on_shuffle_tracks(move || {
                playback::play_tracks(
                    runtime.clone(),
                    weak.clone(),
                    handle.clone(),
                    favorites::shuffled_tracks(),
                    0,
                );
            });
    }
    {
        // Enter / leave the tracks multi-select edit mode.
        let weak = window.as_weak();
        window
            .global::<FavoritesActions>()
            .on_toggle_multi_select(move || {
                if let Some(w) = weak.upgrade() {
                    let on = w.global::<FavoritesState>().get_tracks_multi_select();
                    favorites::set_multi_select(&w, !on);
                }
            });
    }
    {
        // Bulk bar actions over the selected favorite tracks.
        let runtime = app_runtime.clone();
        let weak = window.as_weak();
        let handle = tokio_rt.handle().clone();
        let image_cache = image_cache.clone();
        window
            .global::<FavoritesActions>()
            .on_bulk_action(move |action| {
                let Some(w) = weak.upgrade() else {
                    return;
                };
                match action.as_str() {
                    "select-all" => favorites::select_all(&w),
                    "clear" => favorites::clear_selection(&w),
                    "queue" => {
                        let tracks = favorites::selected_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, false);
                    }
                    "play-next" => {
                        let tracks = favorites::selected_tracks(&w);
                        playback::enqueue_tracks(runtime.clone(), handle.clone(), tracks, true);
                    }
                    "make-offline" => {
                        let tracks = favorites::selected_tracks(&w);
                        offline_cache::cache_tracks(
                            runtime.clone(),
                            weak.clone(),
                            handle.clone(),
                            tracks,
                        );
                        favorites::clear_selection(&w);
                    }
                    "add-to-mixtape" => {
                        let items =
                            mixtape_items_from_qobuz_tracks(&favorites::selected_tracks(&w));
                        if !items.is_empty() {
                            open_add_to_mixtape(weak.clone(), handle.clone(), items);
                            favorites::clear_selection(&w);
                        }
                    }
                    "add-to-playlist" => {
                        let ids = favorites::selected_ids(&w);
                        if !ids.is_empty() {
                            playlist_picker::open_multi(&w, &ids, false);
                            let runtime = runtime.clone();
                            let weak = weak.clone();
                            handle.spawn(async move {
                                let playlists = playlist_picker::load(&runtime).await;
                                let _ = weak.upgrade_in_event_loop(move |w| {
                                    playlist_picker::apply(&w, playlists);
                                });
                            });
                        }
                    }
                    "remove-selected" => {
                        // Offline = read-only hearts (spec 4.3).
                        if offline_mode::engine().is_offline() {
                            toast::info(&w, "Not available offline");
                            return;
                        }
                        let ids = favorites::selected_ids(&w);
                        if ids.is_empty() {
                            return;
                        }
                        let runtime = runtime.clone();
                        let weak = weak.clone();
                        let handle = handle.clone();
                        let image_cache = image_cache.clone();
                        handle.clone().spawn(async move {
                            for id in &ids {
                                if let Err(e) =
                                    runtime.core().remove_favorite("track", id).await
                                {
                                    log::error!(
                                        "[qbz-slint] bulk remove favorite {id} failed: {e}"
                                    );
                                }
                                if let Ok(tid) = id.parse::<u64>() {
                                    crate::fav_cache::set(tid, false);
                                }
                            }
                            let _ = weak.upgrade_in_event_loop(|w| {
                                favorites::set_multi_select(&w, false);
                            });
                            navigate_favorites(
                                runtime.clone(),
                                weak.clone(),
                                &handle,
                                image_cache.clone(),
                                favorites::FavTab::Tracks,
                                "tracks",
                            );
                        });
                    }
                    _ => {}
                }
            });
    }

    // Artwork right-click menu wiring — Open in browser / Save as /
    // Add custom / Remove custom. Mirrors the v2_library_* + native
    // dialog flow Tauri uses on artist portraits + album covers.
    window
        .global::<ArtworkActions>()
        .on_open_in_browser(|url| {
            if url.is_empty() {
                return;
            }
            if let Err(e) = open::that(url.as_str()) {
                log::error!("[qbz-slint] artwork open-in-browser failed: {e}");
            }
        });
    {
        let handle = tokio_rt.handle().clone();
        window
            .global::<ArtworkActions>()
            .on_save_as(move |url, default_name| {
                if url.is_empty() {
                    return;
                }
                let url = url.to_string();
                let default = default_name.to_string();
                handle.spawn(async move {
                    let Some(dest) = rfd::AsyncFileDialog::new()
                        .set_file_name(&default)
                        .add_filter("Images", &["jpg", "jpeg", "png"])
                        .save_file()
                        .await
                    else {
                        return;
                    };
                    // Offline: serve the shared disk-cache copy; never attempt
                    // the download.
                    if offline_mode::engine().is_offline() {
                        match artwork::cached_path_for(&url) {
                            Some(path) => {
                                if let Err(e) = tokio::fs::copy(&path, dest.path()).await {
                                    log::error!(
                                        "[qbz-slint] artwork save-as offline copy: {e}"
                                    );
                                }
                            }
                            None => log::warn!(
                                "[qbz-slint] artwork save-as skipped offline: not in disk cache"
                            ),
                        }
                        return;
                    }
                    let bytes = match reqwest::get(&url).await {
                        Ok(resp) => match resp.bytes().await {
                            Ok(b) => b,
                            Err(e) => {
                                log::error!(
                                    "[qbz-slint] artwork save-as fetch body: {e}"
                                );
                                return;
                            }
                        },
                        Err(e) => {
                            log::error!("[qbz-slint] artwork save-as request: {e}");
                            return;
                        }
                    };
                    if let Err(e) = tokio::fs::write(dest.path(), &bytes).await {
                        log::error!("[qbz-slint] artwork save-as write: {e}");
                    }
                });
            });
    }
    {
        let handle = tokio_rt.handle().clone();
        let weak = window.as_weak();
        let image_cache = image_cache.clone();
        window
            .global::<ArtworkActions>()
            .on_add_custom(move |kind, key| {
                let kind = kind.to_string();
                let key = key.to_string();
                let weak = weak.clone();
                let image_cache = image_cache.clone();
                handle.spawn(async move {
                    let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                        .await
                    else {
                        return;
                    };
                    let path = file.path().to_string_lossy().into_owned();
                    match kind.as_str() {
                        "artist" => {
                            custom_artwork::set_artist_image(&key, &path);
                            // Decode + apply immediately so the new image shows
                            // without a reload — critical for artists with no
                            // Qobuz portrait (e.g. Vicky Psarakis), where there
                            // is no network artwork to fall back on.
                            let decoded = artwork::fetch_and_decode_ref(
                                &qbz_models::ArtworkRef::LocalFile(path.clone()),
                                &image_cache,
                                440,
                            )
                            .await;
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                w.global::<ArtistState>().set_has_custom_image(true);
                                if let Some((pixels, iw, ih)) = decoded {
                                    artist::apply_artwork(&w, &pixels, iw, ih);
                                }
                            });
                        }
                        "album" => {
                            custom_artwork::set_album_cover(&key, &path);
                            let decoded = artwork::fetch_and_decode_ref(
                                &qbz_models::ArtworkRef::LocalFile(path.clone()),
                                &image_cache,
                                448,
                            )
                            .await;
                            let _ = weak.upgrade_in_event_loop(move |w| {
                                w.global::<AlbumState>().set_has_custom_cover(true);
                                if let Some((pixels, iw, ih)) = decoded {
                                    album::apply_artwork(&w, &pixels, iw, ih);
                                }
                            });
                        }
                        _ => log::warn!(
                            "[qbz-slint] artwork add-custom: unknown kind {kind}"
                        ),
                    }
                });
            });
    }
    {
        let weak = window.as_weak();
        window
            .global::<ArtworkActions>()
            .on_remove_custom(move |kind, key| {
                match kind.as_str() {
                    "artist" => {
                        custom_artwork::remove_artist_image(key.as_str());
                        if let Some(w) = weak.upgrade() {
                            w.global::<ArtistState>().set_has_custom_image(false);
                        }
                    }
                    "album" => {
                        custom_artwork::remove_album_cover(key.as_str());
                        if let Some(w) = weak.upgrade() {
                            w.global::<AlbumState>().set_has_custom_cover(false);
                        }
                    }
                    _ => log::warn!(
                        "[qbz-slint] artwork remove-custom: unknown kind {kind}"
                    ),
                }
            });
    }

    window.on_close_app({
        let weak = window.as_weak();
        move || {
            // Custom titlebar close button. Hide to tray when close-to-tray is
            // enabled and the tray is live; otherwise quit.
            if tray_settings::get().close_to_tray && tray::handle().is_some() {
                log::info!("[qbz-slint] close-to-tray (titlebar): hiding to tray");
                // Flush the session even when only hiding — the process may be
                // killed from the tray / shell without a real quit afterwards.
                session_persist::save_on_exit();
                tray::hide_window(&weak);
            } else {
                log::info!("[qbz-slint] closing");
                // Flush the final session snapshot before quitting.
                session_persist::save_on_exit();
                let _ = slint::quit_event_loop();
            }
        }
    });

    // Intercept the window-manager close (native titlebar X / compositor
    // close). Mirrors the custom titlebar: hide to tray when close-to-tray is
    // on + the tray is live, otherwise quit. Required because the loop runs
    // with quit_on_last_window_closed = false (so a tray-hide keeps the app
    // alive) — without this, the native close would leave a headless process.
    window.window().on_close_requested(move || {
        let settings = tray_settings::get();
        if settings.close_to_tray && tray::handle().is_some() {
            // Slint performs the hide (destroys the surface) for HideWindow;
            // we only sync the shown flag so the next tray toggle shows it.
            log::info!("[qbz-slint] close-to-tray (WM close): hiding to tray");
            // Flush the session even when only hiding — the process may be
            // killed from the tray / shell without a real quit afterwards.
            session_persist::save_on_exit();
            tray::set_window_shown(false);
            // macOS: drop the Dock icon if the user opted in (no-op elsewhere).
            if settings.mac_hide_dock {
                tray::set_mac_dock_hidden(true);
            }
            slint::CloseRequestResponse::HideWindow
        } else {
            log::info!("[qbz-slint] WM close requested: quitting");
            // Flush the final session snapshot before quitting.
            session_persist::save_on_exit();
            let _ = slint::quit_event_loop();
            slint::CloseRequestResponse::HideWindow
        }
    });

    window.on_open_tos(|| {
        dispatch(AppCommand::OpenTermsOfService);
        if let Err(e) = open::that(QOBUZ_TOS_URL) {
            log::error!("[qbz-slint] failed to open Terms of Service: {e}");
        }
    });

    log::info!("[qbz-slint] window ready");
    // NOT `window.run()`: that quits the event loop when the last window
    // closes, which would kill the app the moment the window hides to tray.
    // `run_event_loop_until_quit()` keeps the loop alive until an explicit
    // `quit_event_loop()` (custom titlebar / WM close when not close-to-tray /
    // tray Quit), so hide-to-tray works.
    window.show()?;
    // macOS custom chrome: centre the native traffic lights in the 42px
    // header — AppKit parks them at the stock ~28pt titlebar height, visibly
    // above the header controls. Queued so it runs after the event loop has
    // processed the show (the NSWindow/handle only exists then).
    #[cfg(target_os = "macos")]
    {
        let weak = window.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(w) = weak.upgrade() {
                macos_chrome::center_traffic_lights(w.window());
            }
        });
    }
    slint::run_event_loop_until_quit()?;
    // Reaching here = a CLEAN exit (tray Quit calls quit_event_loop directly,
    // with no window input event) — without this, launch-then-quit-from-tray
    // inside the 30s fallback would leave the sentinel armed and falsely
    // walk the renderer ladder on the next start.
    disarm_renderer_sentinel_on_liveness("clean exit");
    // Single choke point for ALL quit paths (custom-titlebar close, WM close,
    // tray Quit): release anything QBZ parked on the audio graph before the
    // process exits. Quitting mid-playback never runs the audio thread's Stop
    // handler, so a forced PipeWire clock (DAC passthrough) outlived the app
    // and pinned every other program to the last track's sample rate until
    // PipeWire restarted (#521). Both calls are self-gating no-ops when QBZ
    // didn't set anything — same pair the Stop/ReleaseDevice handlers use.
    #[cfg(target_os = "linux")]
    {
        qbz_audio::alsa_backend::resume_suspended_sink();
        qbz_audio::pipewire_backend::PipeWireBackend::reset_pipewire_clock();
    }
    Ok(())
}

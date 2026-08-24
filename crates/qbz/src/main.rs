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
//!
//! DELIBERATE EXCEPTION to the 130-line budget (crates/qbz/src/main.rs
//! refactor), for two independent, unrelated reasons:
//!
//! 1. The `mod x;` declaration list below (~120 lines) is the crate root's
//!    module tree. Rust has no mechanism to declare `mod x;` from a
//!    non-root file and have `x` resolve as `crate::x` — every declaration
//!    must live textually in the file that IS the intended parent module.
//!    Moving them elsewhere would require either rewriting every
//!    `crate::x::y` reference in the whole crate to a new path, or reaching
//!    for `include!()` to splice another file's tokens in place (a rarer,
//!    harder-to-grep pattern not worth adopting for a flat, order-
//!    insensitive declaration list that isn't actually hard to read).
//! 2. `fn main()`'s own body has a further, already-analyzed split (see the
//!    inline comment at the boundary below): everything through the
//!    sign-in wiring is a strictly sequential imperative boot procedure
//!    that creates several long-lived handles — most importantly the tokio
//!    `Runtime` and its `_enter` guard, which must stay alive in this exact
//!    stack frame for `Handle::current()` calls (here and in every spawned
//!    task) to keep working. Decomposing it further without a compiler in
//!    the loop (no `cargo check` is permitted for this refactor — see
//!    refactor-plans/crates__qbz__src__main.rs.md) risks silently breaking
//!    app startup, the single most safety-critical path in the binary.
//!    Everything AFTER that boundary (the `wire_*` cluster calls) is
//!    already minimal and cluster-split; see each `wire_*` module.

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

// `fn main()`'s own ~12,150-line body (formerly all inline) is split the
// same way, except its imperative startup sequence (window/runtime/cache
// creation — see the note at its call site in `fn main()` below) which
// stays inline for a genuine Rust-level ordering/lifetime reason.
mod wire_offline_and_auth;
mod wire_search;
mod wire_link_and_import;
mod wire_home_library_playback;
mod wire_queue_and_cards;
mod wire_info_modals_suggestions;
mod wire_discover_offline_manager;
mod wire_local_library_settings;
mod wire_playlist_browse_picker;
mod wire_playlist_crud_sidebar;
mod wire_create_playlist_dac_import;
mod wire_library_all_artwork_close;

pub(crate) use wire_offline_and_auth::*;
pub(crate) use wire_search::*;
pub(crate) use wire_link_and_import::*;
pub(crate) use wire_home_library_playback::*;
pub(crate) use wire_queue_and_cards::*;
pub(crate) use wire_info_modals_suggestions::*;
pub(crate) use wire_discover_offline_manager::*;
pub(crate) use wire_local_library_settings::*;
pub(crate) use wire_playlist_browse_picker::*;
pub(crate) use wire_playlist_crud_sidebar::*;
pub(crate) use wire_create_playlist_dac_import::*;
pub(crate) use wire_library_all_artwork_close::*;

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

    // --- main.rs split (crates/qbz/src/main.rs refactor) -------------------
    // Everything above this point (from `fn main()`'s signature through the
    // sign-in wiring just above) is left INLINE on purpose: it's the
    // imperative, tightly sequential boot procedure that CREATES the
    // long-lived handles below (`window`, `tokio_rt` — whose `_enter` guard
    // must stay alive in this exact stack frame for `Handle::current()` to
    // keep working — `app_runtime`, `image_cache`, `settings_ctx`), so it
    // cannot be moved into a called function without either dropping the
    // tokio "entered" guard early or fighting a self-referential-borrow
    // problem. See refactor-plans/crates__qbz__src__main.rs.md (cluster 14)
    // and the wire_startup_sequence note in this file's module doc.
    //
    // Everything from here to the end of `fn main()` is the ~370-callback
    // Slint wiring section, which — once past this point — only ever reads
    // the five handles above (verified: no further shared `let` bindings
    // appear in the original body after this point). It is split into 12
    // sibling `wire_*` clusters, called here in their original relative
    // order, each further split into ≤130-line `wire_x_partN` files.
    wire_offline_and_auth(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_search(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_link_and_import(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_home_library_playback(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_queue_and_cards(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_info_modals_suggestions(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_discover_offline_manager(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_local_library_settings(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_playlist_browse_picker(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_playlist_crud_sidebar(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_create_playlist_dac_import(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);
    wire_library_all_artwork_close(&window, &app_runtime, &tokio_rt, &image_cache, &settings_ctx);

    Ok(())
}

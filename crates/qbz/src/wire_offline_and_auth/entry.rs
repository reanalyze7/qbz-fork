use crate::*;

pub(crate) fn wire_offline_and_auth(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {
    // Sign in via the system browser → real OAuth → shell. The app has no
    // embedded webview: the one blue button opens the default browser, and
    // LoginState narrates the flow (waiting / authenticating / error) so the
    // login screen never sits inert while the OAuth is pending.
    // The in-flight task is kept so the screen's Cancel link can abort it
    // (dropping the task drops the one-shot listener and frees the port).
    //
    // Created here (not inside a `part*` fn) because it is shared by
    // `wire_offline_and_auth_part1` (creates the browser-login closure that
    // populates it) and `_part2` (the D2 recovery-login banner button) — a
    // genuine cross-callback ordering dependency from the original
    // `fn main()` body, threaded through explicitly rather than re-created
    // per part (which would silently give each part its own independent
    // mutex and break Cancel).
    let login_task: Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>> =
        Arc::new(std::sync::Mutex::new(None));
    wire_offline_and_auth_part1(window, app_runtime, tokio_rt, image_cache, settings_ctx, &login_task);
    wire_offline_and_auth_part2(window, app_runtime, tokio_rt, image_cache, settings_ctx, &login_task);
    wire_offline_and_auth_part3(window, app_runtime, tokio_rt, image_cache, settings_ctx);
    wire_offline_and_auth_part4(window, app_runtime, tokio_rt, image_cache, settings_ctx);
}

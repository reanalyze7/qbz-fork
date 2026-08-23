//! Resolve + navigate via the existing Ctrl+L link-resolver flow.

use slint::ComponentHandle;

use super::shell_ctx::ShellCtx;

/// Mirror of the Ctrl+L `LinkResolverActions::on_submit` flow in `main.rs`,
/// minus the modal state: resolve off the UI thread, then apply the
/// navigation on the event loop. Failures surface as a toast (there is no
/// modal to hold an error here).
pub(super) fn dispatch(url: String, ctx: ShellCtx) {
    let ShellCtx {
        runtime,
        weak,
        handle,
        image_cache,
    } = ctx;
    log::info!(
        "[qbz-slint] deep link: resolving {}",
        url.split('?').next().unwrap_or(&url)
    );
    handle.clone().spawn(async move {
        let result = crate::link_resolver::resolve(runtime.clone(), url).await;
        let _ = weak.upgrade_in_event_loop(move |w| match result {
            Ok(qbz_music_link::MusicLinkResult::Resolved { link, .. }) => {
                crate::apply_resolved_link(link, &runtime, &w.as_weak(), &handle, &image_cache);
            }
            Ok(qbz_music_link::MusicLinkResult::PlaylistDetected { provider }) => {
                // Unreachable for the native Qobuz shapes the argv/D-Bus
                // matcher accepts (cross-platform provider playlists only).
                log::info!("[qbz-slint] deep link: {provider} playlist — nothing to navigate to");
            }
            Ok(qbz_music_link::MusicLinkResult::NotOnQobuz { .. }) => {
                crate::toast::error(
                    &w,
                    qbz_i18n::t("This content is not available on Qobuz"),
                );
            }
            Err(e) => {
                log::warn!("[qbz-slint] deep link: resolve failed: {e}");
                crate::toast::error(&w, qbz_i18n::t("Could not resolve that link"));
            }
        });
    });
}

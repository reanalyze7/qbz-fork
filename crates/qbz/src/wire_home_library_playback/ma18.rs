use crate::*;

// One batch of `on_media_action` match arms, split out of the original
// single 2087-line callback (crates/qbz/src/main.rs refactor) to stay
// under the 130-line file cap. Called unconditionally in original arm
// order from `dispatch_media_action` (ma_dispatch.rs); each batch's
// `match` only fires for its own (kind, action) patterns, no-op otherwise.
pub(crate) fn ma_batch18(
    kind: &str,
    id: &str,
    action: &str,
    runtime: &Arc<AppRuntime<SlintAdapter>>,
    weak: &slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: &artwork::ImageCache,
) {
    let runtime = runtime.clone();
    let weak = weak.clone();
    let handle = handle.clone();
    let image_cache = image_cache.clone();
    let id = id.to_string();
    match (kind, action) {
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
        _ => {}
    }
}

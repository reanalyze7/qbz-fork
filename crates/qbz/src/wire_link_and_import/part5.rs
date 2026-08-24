use crate::*;

pub(crate) fn wire_link_and_import_part5(window: &AppWindow, app_runtime: &Arc<AppRuntime<SlintAdapter>>, tokio_rt: &tokio::runtime::Runtime, image_cache: &artwork::ImageCache, settings_ctx: &Arc<settings::SettingsCtx>) {

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
}

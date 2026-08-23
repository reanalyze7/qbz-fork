//! Modal open/close: the payload + header-string computation.

use slint::{ComponentHandle, ModelRc, VecModel};

use super::{AddItem, PENDING};
use crate::{AppWindow, MyQbzAddRow, MyQbzAddState};

/// Open the picker for one or more items. Empty input is a no-op (mirrors
/// `openAddToMixtape([])`). Stores the payload, computes the header strings +
/// kind-restriction, marks loading, and shows the modal. UI thread.
///
/// The caller is responsible for spawning the row load afterwards (it needs the
/// tokio handle + a worker thread for the DB read); the wiring in `main.rs`
/// does `open(...)` then spawns [`super::load_rows`] → [`super::apply_rows`].
pub fn open(window: &AppWindow, items: Vec<AddItem>) {
    if items.is_empty() {
        return;
    }

    let bulk = items.len() > 1;
    // restrict to mixtapes if ANY pending item is a track/playlist (collections
    // hold whole albums only).
    let restrict = items.iter().any(|it| it.item_type != "album");

    let first = &items[0];
    let header_title: String = if bulk {
        qbz_i18n::tf("{} item", "{} items", items.len() as i64, &[&items.len().to_string()])
    } else {
        first.title.clone()
    };
    let header_subtitle: String = if bulk {
        // "{first title} + N more" — 1:1 with PSD.
        let more = (items.len() - 1).to_string();
        qbz_i18n::t_args("{} + {} more", &[&first.title, &more])
    } else {
        first.subtitle.clone().unwrap_or_default()
    };

    if let Ok(mut p) = PENDING.lock() {
        *p = items;
    }

    let state = window.global::<MyQbzAddState>();
    state.set_rows(ModelRc::new(VecModel::from(Vec::<MyQbzAddRow>::new())));
    state.set_header_title(header_title.into());
    state.set_header_subtitle(header_subtitle.into());
    state.set_bulk_mode(bulk);
    state.set_restrict_to_mixtape(restrict);
    state.set_search("".into());
    state.set_busy_id("".into());
    state.set_creating(false);
    state.set_create_name("".into());
    state.set_create_kind("mixtape".into());
    state.set_loading(true);
    state.set_open(true);
}

/// Close the picker + clear the pending payload. UI thread.
pub fn close(window: &AppWindow) {
    if let Ok(mut p) = PENDING.lock() {
        p.clear();
    }
    let state = window.global::<MyQbzAddState>();
    state.set_open(false);
    state.set_creating(false);
    state.set_busy_id("".into());
}

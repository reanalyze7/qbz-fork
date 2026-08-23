//! Local-favorite toggle for an album card.

use slint::{ComponentHandle, Model};

use crate::AppWindow;

use super::basics::for_each_album_model;

/// Toggle the local-favorite state of a local album by its composite key.
/// Reads the album's display snapshot from the rendered model, writes to the
/// local-favorites store (genuine local files only — offline-cache albums are
/// skipped), and optimistically flips the heart on every rendered model.
pub fn toggle_album_favorite(window: &AppWindow, id: &str) {
    let mut snap: Option<(String, String, String, String)> = None; // title, artist, artwork, source
    for_each_album_model(window, |m| {
        if snap.is_some() {
            return;
        }
        for i in 0..m.row_count() {
            if let Some(a) = m.row_data(i) {
                if a.id.as_str() == id {
                    snap = Some((
                        a.title.to_string(),
                        a.artist.to_string(),
                        a.artwork_url.to_string(),
                        a.source.to_string(),
                    ));
                    return;
                }
            }
        }
    });
    let Some((title, artist, artwork_url, source)) = snap else {
        return;
    };
    // Only genuine local files are locally favoritable (never the Qobuz
    // offline cache).
    if source != "local" {
        return;
    }
    let item = crate::local_favorites::LocalFavItem {
        kind: "album".into(),
        id: id.to_string(),
        title,
        subtitle: String::new(),
        artwork_url,
        artist,
        source,
        favorited_at: 0,
    };
    let new_state = crate::local_favorites::toggle(&item).unwrap_or(false);
    for_each_album_model(window, |m| {
        for i in 0..m.row_count() {
            if let Some(mut a) = m.row_data(i) {
                if a.id.as_str() == id && a.is_favorite != new_state {
                    a.is_favorite = new_state;
                    m.set_row_data(i, a);
                }
            }
        }
    });
}

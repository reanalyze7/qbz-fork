//! Async GitHub avatar fetch + paint (author + contributors).

use slint::{ComponentHandle, Model};

use crate::{AboutState, AppWindow};

use super::meta::{AUTHOR_HANDLE, CONTRIBUTORS, CONTRIBUTORS_PER_ROW};

/// The GitHub avatar URL for a handle (64px PNG, matching the Tauri build).
fn avatar_url(handle: &str) -> String {
    format!("https://github.com/{handle}.png?size=64")
}

/// Fetch every GitHub avatar (author + contributors) off the UI thread and paint
/// each onto its own chip as it arrives. A failed fetch just leaves that chip's
/// blank circle in place — no crash, no retry.
pub(super) fn spawn_avatar_loads(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("qbz")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[qbz-slint] about avatar client build failed: {e}");
            return;
        }
    };

    // Author avatar → its dedicated AboutState field.
    {
        let weak = weak.clone();
        let client = client.clone();
        let url = avatar_url(AUTHOR_HANDLE);
        handle.spawn(async move {
            if let Some((pixels, w, h)) = fetch_avatar(&client, &url).await {
                let _ = weak.upgrade_in_event_loop(move |win| {
                    let img = crate::artwork::pixels_to_image(&pixels, w, h);
                    win.global::<AboutState>().set_author_avatar(img);
                });
            }
        });
    }

    // Contributor avatars → addressed by (group, position) in the grouped model.
    for (idx, contributor) in CONTRIBUTORS.iter().enumerate() {
        let weak = weak.clone();
        let client = client.clone();
        let url = avatar_url(contributor);
        let group = idx / CONTRIBUTORS_PER_ROW;
        let pos = idx % CONTRIBUTORS_PER_ROW;
        handle.spawn(async move {
            if let Some((pixels, w, h)) = fetch_avatar(&client, &url).await {
                let _ = weak.upgrade_in_event_loop(move |win| {
                    let img = crate::artwork::pixels_to_image(&pixels, w, h);
                    let groups = win.global::<AboutState>().get_contributor_rows();
                    if let Some(grp) = groups.row_data(group) {
                        let items = grp.items.clone();
                        if let Some(mut row) = items.row_data(pos) {
                            row.avatar = img;
                            items.set_row_data(pos, row);
                        }
                    }
                });
            }
        });
    }
}

/// Fetch one GitHub avatar and decode it to RGBA8 (downscaled to 64px). `None`
/// on any network/decode failure — the caller leaves the blank circle.
async fn fetch_avatar(
    client: &reqwest::Client,
    url: &str,
) -> Option<(Vec<u8>, u32, u32)> {
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    let rgba = image::load_from_memory(&bytes).ok()?.thumbnail(64, 64).to_rgba8();
    let (w, h) = rgba.dimensions();
    Some((rgba.into_raw(), w, h))
}

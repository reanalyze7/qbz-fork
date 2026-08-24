use crate::*;

// Fetch + apply the artist portrait: a user-set custom portrait wins (and
// is the only source for artists with no Qobuz image); otherwise fall back
// to the Qobuz artwork_url. Split out of `navigate_artist` (part3.rs) to
// stay under the 130-line file cap — pure extraction, awaited in place of
// the original inline `if`/`else if`.
pub(crate) async fn apply_artist_portrait(
    weak: slint::Weak<AppWindow>,
    image_cache: artwork::ImageCache,
    custom_image_path: Option<String>,
    artwork_url: String,
) {
                if let Some(path) = custom_image_path {
                    // User-set custom portrait wins (and is the only source
                    // for artists with no Qobuz image).
                    if let Some((pixels, width, height)) = artwork::fetch_and_decode_ref(
                        &qbz_models::ArtworkRef::LocalFile(path),
                        &image_cache,
                        440,
                    )
                    .await
                    {
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            artist::apply_artwork(&w, &pixels, width, height);
                        });
                    }
                } else if !artwork_url.is_empty() {
                    if let Some((pixels, width, height)) =
                        artwork::fetch_and_decode(&artwork_url, &image_cache, 440).await
                    {
                        let _ = weak.upgrade_in_event_loop(move |w| {
                            artist::apply_artwork(&w, &pixels, width, height);
                        });
                    }
                }
}

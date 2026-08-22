//! Pure helpers for `search_artwork_options`: query building and result triage.

use super::types::{DiscogsImageOption, SearchResult};

/// Build the search query, preferring a catalog number when one is given.
pub(super) fn build_artwork_query(artist: &str, album: &str, catalog_number: Option<&str>) -> String {
    match catalog_number.filter(|s| !s.trim().is_empty()) {
        Some(catno) => catno.to_string(),
        None => format!("{} {}", artist, album),
    }
}

/// Split search results into the top 2 release/master IDs (for detailed image
/// fetch) and the remaining release/master results (fallback images).
pub(super) fn split_top_releases(results: &[SearchResult]) -> (Vec<u64>, Vec<&SearchResult>) {
    let mut release_ids: Vec<u64> = Vec::new();
    let mut other_results: Vec<&SearchResult> = Vec::new();

    for result in results.iter().take(20) {
        if result.result_type == "release" || result.result_type == "master" {
            if release_ids.len() < 2 {
                release_ids.push(result.id);
            } else {
                other_results.push(result);
            }
        }
    }

    (release_ids, other_results)
}

/// Add up to 2 more images from the non-top-2 search results (cover image
/// preferred, falling back to thumbnail), stopping once 10 images total exist.
pub(super) fn collect_other_result_images(
    other_results: &[&SearchResult],
    all_images: &mut Vec<DiscogsImageOption>,
    seen_urls: &mut std::collections::HashSet<String>,
) {
    for result in other_results.iter().take(10) {
        if all_images.len() >= 10 {
            break;
        }

        // Prefer cover image
        let image_url = if let Some(cover) = &result.cover_image {
            if !cover.is_empty() && !cover.contains("spacer.gif") {
                Some((cover.clone(), 600, 600, "primary".to_string()))
            } else {
                None
            }
        } else {
            None
        };

        let image_url = image_url.or_else(|| {
            result.thumb.as_ref().and_then(|thumb| {
                if !thumb.is_empty() && !thumb.contains("spacer.gif") {
                    Some((thumb.clone(), 150, 150, "secondary".to_string()))
                } else {
                    None
                }
            })
        });

        if let Some((url, width, height, img_type)) = image_url {
            if seen_urls.insert(url.clone()) {
                all_images.push(DiscogsImageOption {
                    url,
                    width,
                    height,
                    image_type: img_type,
                    release_title: Some(result.title.clone()),
                    release_year: None,
                });
            }
        }
    }
}

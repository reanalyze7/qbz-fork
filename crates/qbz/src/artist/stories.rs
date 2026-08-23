//! Magazine/Stories teaser load + apply.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::ArtistStoryItem;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{ArtistState, AppWindow, StoryItem};

/// One Magazine/Stories teaser for the sidebar.
pub struct StoryData {
    pub title: String,
    pub author: String,
    pub excerpt: String,
    pub url: String,
    pub image_url: String,
}

pub(crate) fn map_story(item: ArtistStoryItem) -> StoryData {
    let author = item
        .authors
        .and_then(|list| list.into_iter().next())
        .map(|a| a.name)
        .unwrap_or_default();
    // `image` is a ready-to-use arc-cdn URL; fall back to the first `images[]`.
    let image_url = item
        .image
        .or_else(|| {
            item.images
                .and_then(|list| list.into_iter().next())
                .map(|img| img.url)
        })
        .unwrap_or_default();
    StoryData {
        url: format!("https://play.qobuz.com/magazine/story/{}", item.id),
        // Magazine content comes from a CMS: titles carry entities
        // (&amp; …), excerpts may additionally carry markup.
        title: crate::strip_html::decode_html_entities(&item.title),
        author,
        excerpt: item
            .description_short
            .as_deref()
            .map(crate::strip_html::strip_html)
            .unwrap_or_default(),
        image_url,
    }
}

/// Fetch the artist's Magazine stories (limit 2, like the official client).
/// Returns an empty list on any failure (the section just stays hidden).
pub async fn load_stories<A>(runtime: &Arc<AppRuntime<A>>, artist_id: &str) -> Vec<StoryData>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let Ok(id) = artist_id.parse::<u64>() else {
        return Vec::new();
    };
    match runtime.core().get_artist_story(id, 0, 2).await {
        Ok(resp) => resp.items.into_iter().map(map_story).collect(),
        Err(e) => {
            log::warn!("[qbz-slint] artist story load failed: {e}");
            Vec::new()
        }
    }
}

/// Apply fetched stories to the sidebar Magazine tab. Returns artwork jobs
/// for the thumbnails (caller spawns them). UI thread.
pub fn apply_stories(window: &AppWindow, stories: Vec<StoryData>) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    let items: Vec<StoryItem> = stories
        .into_iter()
        .enumerate()
        .map(|(index, s)| {
            if !s.image_url.is_empty() {
                jobs.push(ArtworkJob {
                    target: ArtworkTarget::ArtistStory { index },
                    url: s.image_url.clone(),
                });
            }
            StoryItem {
                title: s.title.into(),
                author: s.author.into(),
                excerpt: s.excerpt.into(),
                url: s.url.into(),
                image_url: s.image_url.into(),
                image: slint::Image::default(),
            }
        })
        .collect();
    let st = window.global::<ArtistState>();
    st.set_stories(ModelRc::new(VecModel::from(items)));
    st.set_stories_loading(false);
    jobs
}

//! `load_mix`: resolve a mix kind into its track list.

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::Track;

use super::seed::{build_tracks_to_analyse, mix_listened_seed_ids};
use super::state::shuffle;

pub async fn load_mix<A>(runtime: &AppRuntime<A>, kind: &str) -> Vec<Track>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    match kind {
        "daily" | "weekly" => {
            // Tauri buildSeeds parity: seed listened_tracks_ids from recent plays
            // + favorites (~120), build a track_to_analysed payload from ~9 spread
            // seeds for the PRIMARY algorithm, and fall back to the empty-analysis
            // call when the primary returns nothing. DailyQ vs WeeklyQ differ only
            // by cache bucket (see a3), not by the request.
            let seeds = mix_listened_seed_ids(runtime).await;
            if seeds.is_empty() {
                log::warn!(
                    "[qbz-slint] mix '{kind}': no Qobuz seed tracks (recents + favorites empty) — empty mix"
                );
                Vec::new()
            } else {
                let analysed = build_tracks_to_analyse(runtime, &seeds).await;
                let limit = (50usize.saturating_sub(analysed.len())).max(1) as u32;
                let tracks = match runtime
                    .core()
                    .get_dynamic_suggest_full(&seeds, &analysed, limit)
                    .await
                {
                    Ok(tracks) if !tracks.is_empty() => tracks,
                    Ok(_) => {
                        // FALLBACK (Tauri): retry with empty analysis + limit 50.
                        runtime
                            .core()
                            .get_dynamic_suggest(&seeds, 50)
                            .await
                            .unwrap_or_default()
                    }
                    Err(e) => {
                        log::warn!("[qbz-slint] mix '{kind}': dynamic/suggest failed: {e}");
                        Vec::new()
                    }
                };
                log::info!(
                    "[qbz-slint] mix '{kind}': {} seeds, {} analysed -> {} tracks",
                    seeds.len(),
                    analysed.len(),
                    tracks.len()
                );
                tracks
            }
        }
        "fav" => {
            let mut tracks = favorite_tracks(runtime).await;
            shuffle(&mut tracks);
            tracks
        }
        "top" => playlist_tracks(runtime).await,
        _ => Vec::new(),
    }
}

pub(super) async fn favorite_tracks<A>(runtime: &AppRuntime<A>) -> Vec<Track>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    match runtime.core().get_favorites("tracks", 200, 0).await {
        Ok(value) => qbz_models::lenient::parse_items_array(&value, "tracks", "mix favorite track"),
        Err(_) => Vec::new(),
    }
}

async fn playlist_tracks<A>(runtime: &AppRuntime<A>) -> Vec<Track>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let Ok(playlists) = runtime.core().get_user_playlists().await else {
        return Vec::new();
    };
    let mut out: Vec<Track> = Vec::new();
    for pl in playlists.into_iter().take(5) {
        if out.len() >= 100 {
            break;
        }
        if let Ok(full) = runtime.core().get_playlist(pl.id).await {
            if let Some(container) = full.tracks {
                out.extend(container.items);
            }
        }
    }
    out.truncate(100);
    out
}

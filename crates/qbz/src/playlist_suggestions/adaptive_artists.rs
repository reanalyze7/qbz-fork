//! Pure adaptive seed-artist selection algorithm, ported 1:1 from the Svelte
//! `extractAdaptiveArtists` (quantity scales with playlist size; a 60/40
//! top-frequency/random mix for coherence + discovery; the final selection
//! shuffled).

use qbz_models::Track;

pub(super) fn normalize(s: &str) -> String {
    s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn make_key(title: &str, artist: &str) -> String {
    format!("{}|{}", normalize(title), normalize(artist))
}

pub(super) fn mmss(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Deterministic splitmix64 step (qbz-radio's RNG family) — used for the
/// adaptive-artist shuffle so the seed selection is varied but reproducible per
/// playlist (no `rand` pulled into this hot path).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut state = seed ^ 0xD1B5_4A32_D192_ED03;
    for i in (1..items.len()).rev() {
        let j = (splitmix64(&mut state) % (i as u64 + 1)) as usize;
        items.swap(i, j);
    }
}

/// Adaptive seed-artist selection — 1:1 with the Svelte `extractAdaptiveArtists`
/// (quantity scales with playlist size; a 60/40 top-frequency/random mix for
/// coherence + discovery; the final selection shuffled). Keeps the engine's
/// per-artist MusicBrainz resolution bounded on large playlists.
pub(super) fn extract_adaptive_artists(
    tracks: &[Track],
    playlist_id: u64,
) -> Vec<(Option<u64>, String)> {
    // Count tracks per artist name (first-seen qobuz id retained).
    let mut order: Vec<String> = Vec::new();
    let mut counts: std::collections::HashMap<String, (usize, Option<u64>)> =
        std::collections::HashMap::new();
    for track in tracks {
        let Some(performer) = track.performer.as_ref() else {
            continue;
        };
        let name = performer.name.trim();
        if name.is_empty() {
            continue;
        }
        let entry = counts.entry(name.to_string()).or_insert_with(|| {
            order.push(name.to_string());
            (0, Some(performer.id))
        });
        entry.0 += 1;
    }

    let unique = order.len();
    if unique == 0 {
        return Vec::new();
    }

    let n = tracks.len();
    let limit = if n < 15 {
        5.min(n).max(3)
    } else if n < 50 {
        10.min(((n as f64) * 0.3).ceil() as usize)
    } else if n < 100 {
        15.min(((n as f64) * 0.2).ceil() as usize)
    } else {
        20.min(((n as f64) * 0.15).ceil() as usize)
    };
    let actual = limit.min(unique);

    // Sorted (count desc, then first-seen order for stability).
    let mut sorted: Vec<(String, Option<u64>)> = order
        .iter()
        .map(|name| {
            let (_, qid) = counts[name];
            (name.clone(), qid)
        })
        .collect();
    let count_of = |name: &str| counts.get(name).map(|c| c.0).unwrap_or(0);
    sorted.sort_by(|a, b| count_of(b.0.as_str()).cmp(&count_of(a.0.as_str())));

    let to_pair = |(name, qid): (String, Option<u64>)| (qid, name);

    // Few artists: return all, shuffled.
    if unique <= actual {
        let mut all: Vec<(String, Option<u64>)> = sorted;
        shuffle(&mut all, playlist_id);
        return all.into_iter().map(to_pair).collect();
    }

    let top_count = 1.max(((actual as f64) * 0.6).floor() as usize);
    let random_count = actual - top_count;

    let top: Vec<(String, Option<u64>)> = sorted[..top_count].to_vec();
    let mut rest: Vec<(String, Option<u64>)> = sorted[top_count..].to_vec();
    shuffle(&mut rest, playlist_id ^ 0x5EED);
    let random: Vec<(String, Option<u64>)> = rest.into_iter().take(random_count).collect();

    let mut combined: Vec<(String, Option<u64>)> = top.into_iter().chain(random).collect();
    shuffle(&mut combined, playlist_id ^ 0xA5A5);
    combined.into_iter().map(to_pair).collect()
}

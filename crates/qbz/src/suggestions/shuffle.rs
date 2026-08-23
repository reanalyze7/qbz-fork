//! Deterministic splitmix64 RNG for the recommended-track shuffle.

/// Deterministic splitmix64 step (qbz-radio's RNG family). Used for an
/// in-place Fisher-Yates shuffle so the rec list is varied but reproducible
/// per (artist, track) — no `rand` dependency pulled just for this.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Fisher-Yates shuffle seeded off the (artist, track) ids.
pub(super) fn shuffle_tracks(tracks: &mut [qbz_models::Track], seed: u64) {
    let mut state = seed ^ 0xD1B54A32D192ED03;
    for i in (1..tracks.len()).rev() {
        let j = (splitmix64(&mut state) % (i as u64 + 1)) as usize;
        tracks.swap(i, j);
    }
}

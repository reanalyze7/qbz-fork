/// Exponential half-life decay factor for an event `age_secs` old.
pub(super) fn decay_factor(age_secs: i64, half_life_days: f64) -> f64 {
    if half_life_days <= 0.0 {
        return 1.0;
    }
    let half_life_secs = half_life_days * 86_400.0;
    let exponent = age_secs as f64 / half_life_secs;
    0.5_f64.powf(exponent)
}

/// Per-event-type weight (mirrors Tauri's `v2_reco_train_scores` weights).
pub(super) fn event_weight(event_type: &str) -> f64 {
    match event_type {
        "play" => 1.0,
        "favorite" => 3.0,
        "playlist_add" => 1.2,
        _ => 1.0,
    }
}

/// Per-item-type weight: the event's PRIMARY item (the thing it's directly
/// about) always weighs 1.0; secondary items (e.g. the album a track-play
/// also touches) weigh less.
pub(super) fn item_weight(item_type: &str, primary: bool) -> f64 {
    if primary {
        return 1.0;
    }
    match item_type {
        "album" => 0.7,
        "artist" => 0.5,
        "track" => 0.85,
        _ => 0.6,
    }
}

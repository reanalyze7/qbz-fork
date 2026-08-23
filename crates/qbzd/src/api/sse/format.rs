use qbz_models::CoreEvent;

/// Render one CoreEvent as an SSE frame, or `None` for events not worth pushing
/// to a UI client (bulky search payloads, internal loading/download/navigation
/// hints, diagnostics). Everything else — playback, queue, volume, auth,
/// favorites, playlists, errors, device changes — is emitted.
pub(super) fn format_event(ev: &CoreEvent) -> Option<String> {
    if !emit(ev) {
        return None;
    }
    let value = serde_json::to_value(ev).ok()?;
    let typ = value.get("type").and_then(|v| v.as_str()).unwrap_or("event").to_string();
    let data = serde_json::to_string(&value).ok()?;
    Some(format!("event: {typ}\ndata: {data}\n\n"))
}

fn emit(ev: &CoreEvent) -> bool {
    use CoreEvent::*;
    !matches!(
        ev,
        SearchResultsReceived { .. }
            | LoadingStarted { .. }
            | LoadingCompleted { .. }
            | DownloadProgress { .. }
            | DownloadCompleted { .. }
            | NavigateToAlbum { .. }
            | NavigateToArtist { .. }
            | NavigateToPlaylist { .. }
            | AudioDiagnostic { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use qbz_models::PlaybackState;

    #[test]
    fn playback_event_becomes_a_typed_sse_frame() {
        let frame = format_event(&CoreEvent::PlaybackStateChanged {
            state: PlaybackState::Playing,
        })
        .expect("playback event is emitted");
        assert!(frame.starts_with("event: PlaybackStateChanged\n"));
        assert!(frame.contains("data: {"));
        assert!(frame.ends_with("\n\n"));
        // The data line carries the tagged CoreEvent JSON.
        assert!(frame.contains("\"type\":\"PlaybackStateChanged\""));
    }

    #[test]
    fn bulky_and_internal_events_are_not_emitted() {
        assert!(format_event(&CoreEvent::LoadingStarted { operation: "x".into() }).is_none());
        assert!(format_event(&CoreEvent::DownloadCompleted { track_id: 1 }).is_none());
        assert!(format_event(&CoreEvent::NavigateToArtist { artist_id: 1 }).is_none());
    }

    #[test]
    fn volume_and_queue_events_are_emitted() {
        assert!(format_event(&CoreEvent::VolumeChanged { volume: 0.5 }).is_some());
        assert!(format_event(&CoreEvent::ShuffleChanged { enabled: true }).is_some());
    }
}

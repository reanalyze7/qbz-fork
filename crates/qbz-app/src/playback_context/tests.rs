use super::*;

fn album_context() -> PlaybackContext {
    PlaybackContext::new(
        ContextType::Album,
        "album-1".to_string(),
        "Album Title".to_string(),
        ContentSource::Qobuz,
        vec![10, 11, 12, 13],
        1,
    )
}

#[test]
fn playback_context_reports_next_and_upcoming_tracks() {
    let context = album_context();

    assert_eq!(context.next_track_id(), Some(12));
    assert_eq!(context.upcoming_track_ids(2), vec![12, 13]);
    assert_eq!(context.upcoming_track_ids(10), vec![12, 13]);
    assert!(context.has_next());
    assert_eq!(context.total_tracks(), 4);
}

#[test]
fn playback_context_advance_updates_position_until_end() {
    let mut context = album_context();

    assert!(context.advance());
    assert_eq!(context.current_position, 2);
    assert_eq!(context.next_track_id(), Some(13));
    assert!(context.advance());
    assert_eq!(context.current_position, 3);
    assert_eq!(context.next_track_id(), None);
    assert!(!context.has_next());
    assert!(!context.advance());
    assert_eq!(context.current_position, 3);
}

#[test]
fn playback_context_display_info_matches_existing_labels() {
    assert_eq!(album_context().display_info(), "Album · Album Title");

    let radio = PlaybackContext::new(
        ContextType::Radio,
        "radio-1".to_string(),
        "Seed".to_string(),
        ContentSource::Qobuz,
        vec![1],
        0,
    );
    assert_eq!(radio.display_info(), "Radio · Seed");
}

#[test]
fn context_manager_sets_clears_and_reports_context() {
    let manager = ContextManager::new();
    assert!(!manager.has_context());
    assert_eq!(manager.next_track_id(), None);
    assert!(manager.upcoming_track_ids(3).is_empty());
    assert!(!manager.advance_context());

    manager.set_context(album_context());

    assert!(manager.has_context());
    assert_eq!(manager.next_track_id(), Some(12));
    assert_eq!(manager.upcoming_track_ids(3), vec![12, 13]);
    assert_eq!(
        manager.get_context().map(|ctx| ctx.label),
        Some("Album Title".to_string())
    );

    manager.clear_context();
    assert!(!manager.has_context());
}

#[test]
fn context_manager_updates_position_by_track_id() {
    let manager = ContextManager::new();
    manager.set_context(album_context());

    manager.set_position(13);
    let context = manager.get_context().expect("context exists");
    assert_eq!(context.current_position, 3);
    assert_eq!(context.next_track_id(), None);

    manager.set_position(999);
    let context = manager.get_context().expect("context exists");
    assert_eq!(context.current_position, 3);
}

#[test]
fn context_manager_appends_radio_refill_track_ids() {
    let manager = ContextManager::new();
    manager.set_context(album_context());

    manager.append_track_ids(vec![14, 15]);
    let context = manager.get_context().expect("context exists");

    assert_eq!(context.track_ids, vec![10, 11, 12, 13, 14, 15]);
    assert_eq!(context.upcoming_track_ids(10), vec![12, 13, 14, 15]);
}

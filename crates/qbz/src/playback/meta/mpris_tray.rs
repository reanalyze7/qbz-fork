//! Tray tooltip mirror, OS media-controls push, and the desktop "now
//! playing" notification + scrobble edge — all keyed off the same
//! track-change dedupe guards.

use super::fields_types::MetaFields;
use super::statics::{mpris_meta_changed, NOTIFY_LAST_TRACK};
use super::statics::NOTIFICATIONS_ENABLED;
use crate::AppWindow;

/// Mirror `fields` into the tray tooltip, push OS media-controls metadata
/// (de-duped on track id + resolved art), and fire the desktop notification
/// + scrobble on an actual track change (also de-duped).
pub(super) fn sync_mpris_and_tray(_weak: &slint::Weak<AppWindow>, fields: &MetaFields) {
    // Mirror the now-playing metadata into the system tray tooltip (Linux).
    if let Some(t) = crate::tray::handle() {
        t.set_track(fields.title.clone(), fields.artist.clone(), fields.album.clone());
    }

    // LOCAL-FIRST artwork for the desktop NOTIFICATION (B11): remote covers
    // resolve through the shared disk-image cache first — the notify pipeline
    // strips `file://` and decodes the bytes by CONTENT, so the cache's
    // extension-less `<md5>.img` copy is fine there and saves a re-download.
    //
    // MPRIS is different: widgets resolve `mpris:artUrl` file:// URIs through
    // the freedesktop mime database, which maps `*.img` BY EXTENSION to a
    // disk-image type (`application/vnd.efi.img`) — the cached copy is
    // rejected and the widget shows no art (the B11 regression: online plays
    // almost always cache-hit, so every push carried the dead .img URL).
    // ONLINE, MPRIS therefore keeps the remote https URL untouched (widgets
    // fetch it themselves — the production-proven Tauri
    // `normalizeCoverUrlForMetadata` contract). OFFLINE keeps slice-3b's
    // exact semantics: a hit hands MPRIS the file:// copy (nothing else can
    // load — better than no art for widgets that do sniff content), a miss
    // gives no art (widgets can't fetch https), while the notification keeps
    // the remote URL so its own md5 disk cache can still serve it (the
    // offline flag below blocks the download). Local refs keep their
    // normal URL (already file://).
    let offline = crate::offline_mode::engine().is_offline();
    let mut mpris_art = fields.bar_artwork.to_mpris_url();
    let mut notify_art = mpris_art.clone();
    if let qbz_models::ArtworkRef::Remote(url) = &fields.bar_artwork {
        match crate::artwork::cached_file_url_for(url) {
            Some(cached) => {
                notify_art = Some(cached.clone());
                if offline {
                    mpris_art = Some(cached);
                }
            }
            None if offline => {
                mpris_art = None;
            }
            None => {}
        }
    }

    // Push to the OS media controls (MPRIS / SMTC / MediaRemote). The app icon
    // GNOME shows comes from the MPRIS DesktopEntry; `art_url` is the album art
    // (`mpris:artUrl`) — remote covers pass through online (widgets fetch
    // https; never the .img cache copy, see the resolution block above),
    // offline cache hits become a file:// URI. Metadata is de-duped on
    // (track id, resolved art): this refresh re-runs on
    // resume/seek/quality-patch with identical values, so only an actual
    // change re-pushes. `set_playback` stays unconditional.
    if let Some(mc) = crate::media_controls::handle() {
        if mpris_meta_changed(&(fields.track_id_num, mpris_art.clone())) {
            mc.set_metadata(&qbz_media_controls::TrackMeta {
                title: fields.title.clone(),
                artist: fields.artist.clone(),
                album: fields.album_display.clone(),
                duration: (fields.duration > 0)
                    .then(|| std::time::Duration::from_secs(fields.duration)),
                art_url: mpris_art,
            });
        }
        mc.set_playback(
            qbz_media_controls::PlaybackStatus::Playing,
            Some(std::time::Duration::ZERO),
        );
    }

    // Desktop "now playing" notification (1:1 with the Tauri path). De-dupe so
    // only an actual track change fires; skip while a remote QConnect renderer
    // drives playback (matches the Svelte `skipIfRemote`). Fire-and-forget.
    if NOTIFY_LAST_TRACK.swap(fields.track_id_num, std::sync::atomic::Ordering::Relaxed)
        != fields.track_id_num
    {
        let notify_meta = qbz_media_controls::NotificationMeta {
            title: fields.title.clone(),
            artist: fields.artist.clone(),
            album: fields.album_display.clone(),
            bit_depth: fields.bit_depth,
            sample_rate: fields.sample_rate,
            art_url: notify_art,
        };
        // Source-agnostic scrobbling (Last.fm + ListenBrainz). Fires on the
        // SAME de-duped track-change edge as the notification, so resume/seek
        // (which also re-run this fn) do NOT re-fire. Feeds the normalized
        // QueueTrack text (Qobuz, local) with the version-enriched
        // title (#360 parity). Skipped when a remote QConnect renderer drives
        // playback — never scrobble a peer's audio.
        let scrobble_meta = crate::scrobble::ScrobbleMeta {
            artist: fields.artist.clone(),
            track: fields.title.clone(),
            album: (!fields.album.is_empty()).then(|| fields.album.clone()),
            duration_secs: fields.duration,
        };
        tokio::spawn(async move {
            crate::scrobble::on_track_changed(scrobble_meta);
            // Scrobbling above is independent of the notification gate; only the
            // desktop notification honors the System Notifications toggle.
            if NOTIFICATIONS_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
                qbz_media_controls::show_track_notification(notify_meta, offline).await;
            }
        });
    }
}

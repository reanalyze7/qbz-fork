use super::format::build_body;
use super::NotificationMeta;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use super::artwork_cache::cache_artwork;

/// Show a track-change notification. Fire-and-forget: every failure is logged,
/// none propagated. Must be called from within a tokio runtime (it uses
/// `spawn_blocking` for the HTTP/image work). `offline` skips the artwork
/// HTTP download (local paths / disk-cache hits still render an icon).
pub async fn show_track_notification(meta: NotificationMeta, offline: bool) {
    let body = build_body(&meta);
    log::info!(
        "[notify] track notification: {} by {}",
        meta.title,
        meta.artist
    );

    #[cfg(target_os = "linux")]
    {
        use super::linux_icon::prepare_icon_bytes;
        use ashpd::desktop::notification::{Notification as PortalNotification, NotificationProxy};
        use ashpd::desktop::Icon;

        let mut notification =
            PortalNotification::new(&meta.title).body(Some(body.as_str()));

        if let Some(url) = meta.art_url.clone() {
            let prepared = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
                let path = cache_artwork(&url, offline)?;
                prepare_icon_bytes(&path)
            })
            .await;
            match prepared {
                Ok(Ok(bytes)) => {
                    log::debug!("[notify] artwork prepared: {} bytes", bytes.len());
                    notification = notification.icon(Icon::Bytes(bytes));
                }
                Ok(Err(e)) => log::warn!("[notify] could not prepare artwork: {e}"),
                Err(e) => log::warn!("[notify] artwork task failed: {e}"),
            }
        }

        match NotificationProxy::new().await {
            Ok(proxy) => {
                if let Err(e) = proxy
                    .add_notification("track-now-playing", notification)
                    .await
                {
                    log::warn!("[notify] XDG portal add_notification failed: {e}");
                }
            }
            Err(e) => log::warn!("[notify] XDG notification portal unavailable: {e}"),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = tokio::task::spawn_blocking(move || {
            let _ = notify_rust::set_application("com.blitzfc.qbz");
            let artwork_path = meta.art_url.as_deref().and_then(|url| match cache_artwork(url, offline) {
                Ok(path) => Some(path),
                Err(e) => {
                    log::debug!("[notify] could not cache artwork: {e}");
                    None
                }
            });
            let mut notification = notify_rust::Notification::new();
            notification.summary(&meta.title).body(&body);
            if let Some(path) = artwork_path.as_ref().and_then(|p| p.to_str()) {
                notification.image_path(path);
            }
            if let Err(e) = notification.show() {
                log::warn!("[notify] macOS notification failed: {e}");
            }
        })
        .await;
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = body;
        let _ = offline;
        log::info!("[notify] desktop notifications not implemented on this platform");
    }
}

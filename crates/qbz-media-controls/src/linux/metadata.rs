use std::sync::atomic::Ordering;

use mpris_server::{Metadata, PlaybackStatus as MprisStatus, Time, TrackId};

use crate::types::PlaybackStatus;

use super::TRACK_SEQ;

pub(super) fn map_status(s: PlaybackStatus) -> MprisStatus {
    match s {
        PlaybackStatus::Playing => MprisStatus::Playing,
        PlaybackStatus::Paused => MprisStatus::Paused,
        PlaybackStatus::Stopped => MprisStatus::Stopped,
    }
}

pub(super) fn build_metadata(meta: &crate::types::TrackMeta) -> Metadata {
    let seq = TRACK_SEQ.fetch_add(1, Ordering::Relaxed);
    let trackid = TrackId::try_from(format!("/io/github/reanalyze7/qoqobuz/track/{seq}"))
        .unwrap_or(TrackId::NO_TRACK);

    let mut b = Metadata::builder().trackid(trackid).title(meta.title.clone());
    if !meta.artist.is_empty() {
        b = b.artist([meta.artist.clone()]);
    }
    if !meta.album.is_empty() {
        b = b.album(meta.album.clone());
    }
    if let Some(d) = meta.duration {
        b = b.length(Time::from_micros(d.as_micros() as i64));
    }
    if let Some(url) = &meta.art_url {
        b = b.art_url(url.clone());
    }
    b.build()
}

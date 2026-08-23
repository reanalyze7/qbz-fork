//! Shared HTTP streaming feeder.
//!
//! Ports the Tauri `track_loading.rs` progressive feeder verbatim: probe a
//! remote audio URL for size + FLAC format, open the player's progressive
//! streaming sink (`Player::play_streaming_dynamic`), then push the body to the
//! returned `BufferWriter` chunk-by-chunk as it arrives. Playback starts as soon
//! as the initial buffer fills — not after the whole file lands.
//!
//! `reqwest + BufferWriter` bound only, so it stays frontend-side and never
//! crosses the qconnect-app boundary. Used by the QConnect renderer
//! (`qconnect_engine.rs`), so there is exactly one feeder.
//!
//! BIT-PERFECT: `play_streaming_dynamic` decodes the same original bytes and
//! drives the PROTECTED device init from the decoded stream. The
//! `sample_rate`/`bit_depth` parsed here are only the streaming-config hints;
//! the audio backend (`pipewire_backend.rs`, `init_device`, `audio_settings.rs`)
//! is untouched.

use qbz_player::Player;

mod download;
mod errors;
mod probe;

pub use download::download_and_stream_remote_track;
pub use errors::{describe_reqwest_error, is_header_flood_error};
pub use probe::probe_remote_stream_info;

/// Format/size facts sniffed from a remote audio URL before streaming.
pub struct RemoteStreamInfo {
    pub content_length: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u32,
    pub speed_mbps: f64,
}

/// Probe + open the progressive sink + spawn the background feeder.
///
/// On success the player has begun buffering and `play_streaming_dynamic` will
/// start audio once the initial buffer fills; the body download runs in a
/// spawned task. Errors here mean the caller should fall back to a full
/// download (the probe or the sink open failed).
pub async fn stream_remote_track_into_player(
    player: &Player,
    track_id: u64,
    duration_secs: u64,
    start_position_secs: u64,
    url: &str,
    log_tag: &str,
) -> Result<(), String> {
    let stream_info = probe_remote_stream_info(url).await?;
    log::info!(
        "[{}/STREAMING] Track {} - {:.2} MB, {}Hz, {} ch, {}-bit, {:.1} MB/s",
        log_tag,
        track_id,
        stream_info.content_length as f64 / (1024.0 * 1024.0),
        stream_info.sample_rate,
        stream_info.channels,
        stream_info.bit_depth,
        stream_info.speed_mbps
    );

    let writer = player
        .play_streaming_dynamic(
            track_id,
            stream_info.sample_rate,
            stream_info.channels,
            stream_info.bit_depth,
            stream_info.content_length,
            stream_info.speed_mbps,
            duration_secs,
            start_position_secs,
        )
        .map_err(|err| format!("start streaming remote track {track_id}: {err}"))?;

    let url = url.to_string();
    let content_length = stream_info.content_length;
    let log_tag = log_tag.to_string();
    tokio::spawn(async move {
        if let Err(err) =
            download_and_stream_remote_track(&url, writer, track_id, content_length, &log_tag).await
        {
            log::error!(
                "[{}/STREAMING] Track {} failed while streaming: {}",
                log_tag,
                track_id,
                err
            );
        }
    });

    Ok(())
}

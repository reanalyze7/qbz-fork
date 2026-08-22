use super::*;

mod device_control;
mod dsd_dop;
mod dsd_native;
mod dsd_next_dop;
mod gapless;
mod play;
mod play_streaming;
mod resume;
mod seek;
mod transport;

/// Dispatch a single `AudioCommand` to its handler. Extracted from a single
/// `handle_command` closure (see refactor plan) — each variant now lives in
/// its own file taking `&mut ThreadCtx` instead of a dozen `&mut` params.
pub(crate) fn dispatch(ctx: &mut ThreadCtx, command: AudioCommand) {
    match command {
        AudioCommand::Play {
            data,
            track_id,
            duration_secs,
            sample_rate,
            channels,
        } => play::handle(ctx, data, track_id, duration_secs, sample_rate, channels),
        AudioCommand::PlayStreaming {
            source,
            track_id,
            sample_rate,
            channels,
            duration_secs,
            start_position_secs,
            content_length,
            play_gen,
        } => play_streaming::handle(
            ctx,
            source,
            track_id,
            sample_rate,
            channels,
            duration_secs,
            start_position_secs,
            content_length,
            play_gen,
        ),
        AudioCommand::Pause => transport::handle_pause(ctx),
        AudioCommand::Resume => resume::handle(ctx),
        AudioCommand::Stop => transport::handle_stop(ctx),
        AudioCommand::SetVolume(volume) => transport::handle_set_volume(ctx, volume),
        AudioCommand::Seek(position_secs) => seek::handle(ctx, position_secs),
        AudioCommand::ReinitDevice { device_name } => {
            device_control::handle_reinit_device(ctx, device_name)
        }
        AudioCommand::ReleaseDevice => device_control::handle_release_device(ctx),
        AudioCommand::PlayNext {
            data,
            track_id,
            sample_rate,
            channels,
        } => gapless::handle(ctx, data, track_id, sample_rate, channels),
        AudioCommand::PlayDsdDop { path, track_id } => dsd_dop::handle(ctx, path, track_id),
        AudioCommand::PlayDsdNative { path, track_id } => dsd_native::handle(ctx, path, track_id),
        AudioCommand::PlayNextDsdDop { path, track_id } => {
            dsd_next_dop::handle(ctx, path, track_id)
        }
    }
}

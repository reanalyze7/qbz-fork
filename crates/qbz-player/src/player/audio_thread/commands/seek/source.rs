use super::super::super::*;
use super::engine::seek_abort;

/// Build the decoded source for a seek to `position_secs`. Both streaming
/// and cached paths use Symphonia's native seek where possible (FLAC seek
/// table / MP3 TOC), falling back to `decode_with_fallback` +
/// `skip_duration` when Symphonia can't probe the format.
pub(super) fn build_skipped_source(
    ctx: &mut ThreadCtx,
    position_secs: u64,
) -> Option<Box<dyn Source<Item = f32> + Send>> {
    let skip_duration = Duration::from_secs(position_secs);

    if let Some(ref stream_src) = ctx.current_streaming_source {
        return match IncrementalStreamingSource::new(stream_src.clone()) {
            Ok(mut s) => {
                if let Err(e) = s.seek_to(skip_duration) {
                    seek_abort(&ctx.state, &format!("streaming native seek failed: {e}"));
                    return None;
                }
                Some(Box::new(s))
            }
            Err(e) => {
                seek_abort(&ctx.state, &format!("streaming source for seek failed: {e}"));
                None
            }
        };
    }

    let audio_data = ctx
        .current_audio_data
        .as_ref()
        .expect("current_audio_data was checked Some above")
        .clone();

    match InMemorySource::new(audio_data.clone()) {
        Ok(mut s) => match s.seek_to(skip_duration) {
            Ok(()) => Some(Box::new(s)),
            Err(e) => {
                log::warn!(
                    "Native seek on cached source failed ({}); falling back to skip_duration",
                    e
                );
                fallback_skip(ctx, &audio_data, skip_duration)
            }
        },
        Err(e) => {
            log::warn!(
                "InMemorySource probe failed ({}); falling back to skip_duration",
                e
            );
            fallback_skip(ctx, &audio_data, skip_duration)
        }
    }
}

fn fallback_skip(
    ctx: &ThreadCtx,
    audio_data: &[u8],
    skip_duration: Duration,
) -> Option<Box<dyn Source<Item = f32> + Send>> {
    match decode_with_fallback(audio_data) {
        Ok(fb) => Some(Box::new(fb.skip_duration(skip_duration))),
        Err(e) => {
            seek_abort(&ctx.state, &format!("decode for seek failed: {e}"));
            None
        }
    }
}

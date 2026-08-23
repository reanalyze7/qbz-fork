// crates/qbzd/src/cli/settings/write_audio_reinit.rs — the Reinit-class
// `audio.*` write arms (03-setup-tui.md §4.3's 9-field list: closes/reopens
// the output device).

use crate::paths::ProfileRoots;

use super::codec_bool::{parse_alsa_plugin, parse_backend, parse_bool};
use super::codec_value::{parse_dsd_mode, parse_opt_u32, parse_output_device};
use super::store::open_audio;
use super::write::SetError;

/// Returns `Ok(true)` if `key` was one of the Reinit-class audio keys
/// (handled here), `Ok(false)` if not (the caller tries the next domain).
pub(super) fn write_audio_reinit(roots: &ProfileRoots, key: &str, raw: &str) -> Result<bool, SetError> {
    match key {
        "audio.backend" => {
            let v = parse_backend(raw).map_err(SetError::Usage)?;
            open_audio(roots).map_err(SetError::Io)?.set_backend_type(v).map_err(SetError::Io)?
        }
        "audio.device" => {
            let v = parse_output_device(raw);
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_output_device(v.as_deref())
                .map_err(SetError::Io)?
        }
        "audio.alsa_plugin" => {
            let v = parse_alsa_plugin(raw).map_err(SetError::Usage)?;
            open_audio(roots).map_err(SetError::Io)?.set_alsa_plugin(v).map_err(SetError::Io)?
        }
        "audio.alsa_hardware_volume" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_alsa_hardware_volume(v)
                .map_err(SetError::Io)?
        }
        "audio.exclusive_mode" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots).map_err(SetError::Io)?.set_exclusive_mode(v).map_err(SetError::Io)?
        }
        "audio.dac_passthrough" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots).map_err(SetError::Io)?.set_dac_passthrough(v).map_err(SetError::Io)?
        }
        "audio.skip_sink_switch" => {
            let v = parse_bool(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_skip_sink_switch(v)
                .map_err(SetError::Io)?
        }
        "audio.dsd_mode" => {
            let v = parse_dsd_mode(raw).map_err(SetError::Usage)?;
            open_audio(roots).map_err(SetError::Io)?.set_dsd_mode(&v).map_err(SetError::Io)?
        }
        "audio.device_max_sample_rate" => {
            let v = parse_opt_u32(raw).map_err(SetError::Usage)?;
            open_audio(roots)
                .map_err(SetError::Io)?
                .set_device_max_sample_rate(v)
                .map_err(SetError::Io)?
        }
        _ => return Ok(false),
    }
    Ok(true)
}

use qbz_audio::settings::AudioSettings;
use qbz_audio::AudioDevice;

use crate::tui::screens::audio::model::StagedAudio;

pub(super) fn dev(id: &str, name: &str, is_default: bool, is_hardware: bool) -> AudioDevice {
    AudioDevice {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        is_default,
        max_sample_rate: None,
        supported_sample_rates: None,
        device_bus: None,
        is_hardware,
    }
}

pub(super) fn base() -> StagedAudio {
    StagedAudio::from_settings(&AudioSettings::default())
}

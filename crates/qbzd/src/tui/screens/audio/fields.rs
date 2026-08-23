use qbz_audio::AudioBackendType;

use crate::tui::strings as s;

use super::model::StagedAudio;

// ============================ fields + constraint matrix ============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AField {
    Backend,
    Device,
    AlsaPlugin,
    HwVolume,
    Dsd,
    Exclusive,
    Reserve,
    Passthrough,
    ForceBp,
    LockOutput,
    StreamUncached,
    Buffer,
    StreamingOnly,
}

/// Constraint matrix (§3.2.3): `(shown, enabled, disabled_reason)`.
pub fn row_state(field: AField, a: &StagedAudio) -> (bool, bool, Option<&'static str>) {
    use AField::*;
    let alsa = a.backend == AudioBackendType::Alsa;
    let pipewire = a.backend == AudioBackendType::PipeWire;
    match field {
        Backend => (true, true, None),
        Device => (true, true, None),
        AlsaPlugin => (alsa, true, None), // shown only on ALSA
        // NB: `use AField::*` shadows the AlsaPlugin type here — qualify it.
        HwVolume => (alsa && a.alsa_plugin == qbz_audio::AlsaPlugin::Hw, true, None),
        Dsd => (alsa, true, None),
        Exclusive => (true, alsa, if alsa { None } else { Some(s::R_ALSA_ONLY) }),
        Reserve => (true, true, None),
        Passthrough => (
            true,
            pipewire,
            if pipewire { None } else { Some(s::R_PIPEWIRE_ONLY) },
        ),
        // shown only when passthrough on AND PipeWire.
        ForceBp => (a.dac_passthrough && pipewire, true, None),
        // shown when PipeWire; enabled when passthrough OFF.
        LockOutput => (
            pipewire,
            !a.dac_passthrough,
            if a.dac_passthrough { Some(s::R_PASSTHROUGH_OFF) } else { None },
        ),
        StreamUncached => (true, true, None),
        Buffer => (a.stream_first_track, true, None), // shown when stream uncached on
        StreamingOnly => (true, true, None),
    }
}

/// The fields currently SHOWN, top-to-bottom (focus navigates this list).
pub fn visible_fields(a: &StagedAudio) -> Vec<AField> {
    use AField::*;
    [
        Backend, Device, AlsaPlugin, HwVolume, Dsd, Exclusive, Reserve, Passthrough, ForceBp,
        LockOutput, StreamUncached, Buffer, StreamingOnly,
    ]
    .into_iter()
    .filter(|f| row_state(*f, a).0)
    .collect()
}

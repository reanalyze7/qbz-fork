/// `BitPerfectMode` → its serde variant string (02 §3.3.3:
/// `"DirectHardware"|"PluginFallback"|"Disabled"`). `None` = no active stream.
pub(super) fn bitperfect_label(m: Option<qbz_audio::BitPerfectMode>) -> Option<String> {
    use qbz_audio::BitPerfectMode as M;
    m.map(|m| {
        match m {
            M::DirectHardware => "DirectHardware",
            M::PluginFallback => "PluginFallback",
            M::Disabled => "Disabled",
        }
        .to_string()
    })
}

/// Configured backend → the lowercase label the status block shows. `None`
/// (auto-detect) stays `null` until a stream picks a concrete backend.
pub(super) fn backend_label(b: Option<qbz_audio::AudioBackendType>) -> Option<String> {
    use qbz_audio::AudioBackendType as B;
    b.map(|b| {
        match b {
            B::PipeWire => "pipewire",
            B::Alsa => "alsa",
            B::Pulse => "pulse",
            B::Jack => "jack",
            B::SystemDefault => "system",
        }
        .to_string()
    })
}

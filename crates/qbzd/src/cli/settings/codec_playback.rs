// crates/qbzd/src/cli/settings/codec_playback.rs — value parse/render for
// the playback-quality and autoplay-mode keys.

use qbz_app::settings::playback::AutoplayMode;

pub(super) fn parse_streaming_quality(v: &str) -> Result<String, String> {
    match v.to_ascii_lowercase().as_str() {
        "mp3" | "cd" | "hires" | "hires_plus" => Ok(v.to_ascii_lowercase()),
        other => Err(format!(
            "invalid quality '{other}' — expected one of: mp3, cd, hires, hires_plus"
        )),
    }
}

pub(super) fn parse_autoplay(v: &str) -> Result<AutoplayMode, String> {
    serde_json::from_value(serde_json::Value::String(v.to_string())).map_err(|_| {
        format!("invalid autoplay mode '{v}' — expected one of: continue, track_only, infinite")
    })
}
pub(super) fn render_autoplay(mode: AutoplayMode) -> String {
    serde_json::to_value(mode)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "continue".to_string())
}

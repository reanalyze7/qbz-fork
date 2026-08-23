//! Small pure Value-extraction helpers (mirror the Svelte getX helpers).

use serde_json::Value;

/// String for an id-ish Value (string verbatim, number stringified).
pub(super) fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// Display name: `{display}` object form or a bare string.
pub(super) fn name_display(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    v.get("display")
        .and_then(|d| d.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Best URL out of an album `image` Value (string or {large|...}).
pub(super) fn parse_image_value(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    for key in ["large", "extralarge", "medium", "thumbnail", "small"] {
        if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
            return s.to_string();
        }
    }
    String::new()
}

/// getArtistImageUrl: image{large|extralarge|medium|thumbnail|small} ->
/// picture(string) -> images.portrait hash (medium covers).
pub(super) fn parse_artist_image(raw: &Value) -> String {
    if let Some(image) = raw.get("image") {
        let url = parse_image_value(image);
        if !url.is_empty() {
            return url;
        }
    }
    if let Some(pic) = raw.get("picture").and_then(|v| v.as_str()) {
        if !pic.is_empty() {
            return pic.to_string();
        }
    }
    if let Some(portrait) = raw.get("images").and_then(|i| i.get("portrait")) {
        if let (Some(hash), Some(format)) = (
            portrait.get("hash").and_then(|v| v.as_str()),
            portrait.get("format").and_then(|v| v.as_str()),
        ) {
            return format!(
                "https://static.qobuz.com/images/artists/covers/medium/{hash}.{format}"
            );
        }
    }
    String::new()
}

/// getPlaylistImage: image.rectangle -> image.covers[0] ->
/// image{large|thumbnail|small} -> images300[0] -> images150[0] ->
/// images[0].
pub(super) fn parse_playlist_image(raw: &Value) -> String {
    if let Some(image) = raw.get("image") {
        if let Some(rect) = image.get("rectangle").and_then(|v| v.as_str()) {
            if !rect.is_empty() {
                return rect.to_string();
            }
        }
        if let Some(cover) = image
            .get("covers")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
        {
            return cover.to_string();
        }
        for key in ["large", "thumbnail", "small"] {
            if let Some(s) = image.get(key).and_then(|v| v.as_str()) {
                return s.to_string();
            }
        }
    }
    for key in ["images300", "images150", "images"] {
        if let Some(s) = raw
            .get(key)
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
        {
            return s.to_string();
        }
    }
    String::new()
}

/// parseLabelExploreImage: string or {large|thumbnail|small}.
pub(super) fn parse_explore_image(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    for key in ["large", "thumbnail", "small"] {
        if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
            return s.to_string();
        }
    }
    String::new()
}

pub(super) fn mmss(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Word-boundary truncation with an ellipsis (mirrors artist::truncate_words).
pub(super) fn truncate_words(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max).collect();
    let cut = truncated.rfind(' ').unwrap_or(truncated.len());
    format!("{}…", truncated[..cut].trim_end())
}

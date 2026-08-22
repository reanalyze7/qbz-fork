use std::path::PathBuf;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(super) fn artwork_cache_dir() -> Result<PathBuf, String> {
    let dir = dirs::cache_dir()
        .ok_or_else(|| "Could not find cache directory".to_string())?
        .join("qbz")
        .join("artwork");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create artwork cache dir: {e}"))?;
    Ok(dir)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(super) fn resolve_local_artwork(url: &str) -> Option<PathBuf> {
    if let Some(path) = url.strip_prefix("file://") {
        // file:// URLs built with url::Url::from_file_path (e.g. the shared
        // disk-image cache hits handed over by playback) are percent-encoded;
        // decode so paths with spaces/non-ASCII resolve. Fall back to the raw
        // string on invalid UTF-8 escapes (a plain unencoded path).
        let decoded = urlencoding::decode(path)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| path.to_string());
        return Some(PathBuf::from(decoded));
    }
    if let Some(path) = url.strip_prefix("asset://localhost/") {
        let decoded = urlencoding::decode(path).ok()?;
        return Some(PathBuf::from(decoded.into_owned()));
    }
    None
}

/// Shared blocking HTTP client (a fresh client per track leaks an fd → EMFILE
/// over a long session — same reasoning as the Tauri image cache).
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(super) fn http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: std::sync::OnceLock<reqwest::blocking::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .pool_max_idle_per_host(2)
            .build()
            .expect("failed to build notification HTTP client")
    })
}

/// Resolve `url` to a local image file: a `file://`/`asset://` URL maps
/// straight through, an http(s) URL is downloaded and cached by md5(url).
/// `offline` = local paths + md5 cache hits only, never the HTTP download —
/// the verdict is injected by the caller so this crate stays frontend-agnostic
/// (no dependency on the app's offline-mode engine).
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(super) fn cache_artwork(url: &str, offline: bool) -> Result<PathBuf, String> {
    use md5::{Digest, Md5};
    use std::io::Write;

    if let Some(local) = resolve_local_artwork(url) {
        if local.exists() {
            return Ok(local);
        }
    }

    let mut hasher = Md5::new();
    hasher.update(url.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let cache_path = artwork_cache_dir()?.join(format!("{hash}.jpg"));
    if cache_path.exists() {
        return Ok(cache_path);
    }

    if offline {
        return Err("offline: artwork not cached locally".to_string());
    }

    let response = http_client()
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| format!("Failed to download artwork: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to download artwork: HTTP {} (url: {})",
            response.status(),
            url.split('?').next().unwrap_or(url)
        ));
    }
    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read artwork bytes: {e}"))?;
    let mut file =
        std::fs::File::create(&cache_path).map_err(|e| format!("Failed to create cache file: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write artwork cache: {e}"))?;
    Ok(cache_path)
}

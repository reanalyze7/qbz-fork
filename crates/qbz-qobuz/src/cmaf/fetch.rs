use std::sync::Arc;

use super::{CmafProgressCallback, CmafProgressUpdate, CMAF_PREFETCH_CONCURRENCY};

/// Build a reqwest client configured for Akamai CDN fetches.
///
/// Uses the workspace reqwest feature set (rustls-tls). The original in-tree
/// version in `src-tauri/commands_v2/helpers.rs` called `.use_native_tls()`
/// but the src-tauri Cargo opts into both stacks; this crate stays on
/// rustls for smaller binary + no system SSL dependency. If Akamai ever
/// surfaces a cert issue, adding the `native-tls` feature to qbz-qobuz is
/// the escape hatch.
pub(super) fn build_cdn_client() -> std::result::Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("CMAF client error: {}", e))
}

/// Fetch a CDN URL into bytes, retrying transient failures (network blips,
/// 5xx, 429) with exponential backoff. A terminal status (404/403) fails
/// immediately. Without this a single transient segment failure aborted the
/// whole track download and the frontend skipped it — issue #467.
pub(super) async fn fetch_bytes_with_retry(
    http: &reqwest::Client,
    url: &str,
    log_tag: &str,
) -> std::result::Result<Vec<u8>, String> {
    use crate::retry::{
        classify_reqwest, classify_status, retry_transient, FetchError, DEFAULT_MAX_ATTEMPTS,
    };
    retry_transient(
        DEFAULT_MAX_ATTEMPTS,
        log_tag,
        FetchError::is_transient,
        |_attempt| async move {
            let response = http
                .get(url)
                .header("User-Agent", "Mozilla/5.0")
                .send()
                .await
                .map_err(|e| classify_reqwest(&e, "fetch"))?;
            let status = response.status();
            if !status.is_success() {
                return Err(classify_status(status, "fetch"));
            }
            response
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| classify_reqwest(&e, "read"))
        },
    )
    .await
    .map_err(|e| e.to_string())
}

/// Fetch segments 1..=n_segments concurrently with a semaphore cap and a
/// cooldown per slot to stay under CDN rate limits.
///
/// If `on_progress` is `Some`, it's invoked once per completed segment
/// (not per HTTP chunk — the cooldown happens on the worker, not here).
/// Callbacks fire in completion order, not segment order.
pub(super) async fn fetch_all_segments(
    http: &reqwest::Client,
    url_template: &str,
    n_segments: u8,
    log_tag: &str,
    on_progress: Option<CmafProgressCallback>,
) -> std::result::Result<Vec<Vec<u8>>, String> {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(CMAF_PREFETCH_CONCURRENCY));
    let seg_indices: Vec<u8> = (1..=n_segments).collect();
    let mut handles = Vec::with_capacity(seg_indices.len());

    let completed_count = Arc::new(std::sync::atomic::AtomicU32::new(0));

    for seg_idx in seg_indices {
        let sem = semaphore.clone();
        let http = http.clone();
        let seg_url = url_template.replace("$SEGMENT$", &seg_idx.to_string());
        let log_tag = log_tag.to_string();
        let progress = on_progress.clone();
        let counter = completed_count.clone();

        handles.push(tokio::spawn(async move {
            let permit = sem
                .acquire_owned()
                .await
                .map_err(|e| format!("semaphore: {}", e))?;
            let seg_data =
                fetch_bytes_with_retry(&http, &seg_url, &format!("{} seg {}", log_tag, seg_idx))
                    .await
                    .map_err(|e| format!("[{}] seg {} fetch: {}", log_tag, seg_idx, e))?;
            let bytes_this_segment = seg_data.len() as u64;
            if let Some(cb) = progress {
                let done = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                cb(CmafProgressUpdate {
                    segments_completed: done,
                    n_segments: n_segments as u32,
                    bytes_this_segment,
                });
            }
            // Cooldown before releasing the slot — keeps requests spaced out
            // to stay under CDN rate limits (most use 1s windows)
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            drop(permit);
            Ok::<(u8, Vec<u8>), String>((seg_idx, seg_data))
        }));
    }

    // Collect results in arrival order, then re-sort by segment index
    let mut segments: Vec<(u8, Vec<u8>)> = Vec::with_capacity(handles.len());
    for handle in handles {
        let (idx, data) = handle
            .await
            .map_err(|e| format!("[{}] task panic: {}", log_tag, e))?
            .map_err(|e| format!("[{}] download failed: {}", log_tag, e))?;
        segments.push((idx, data));
    }
    segments.sort_by_key(|(idx, _)| *idx);
    Ok(segments.into_iter().map(|(_, data)| data).collect())
}

//! HEAD + ranged GET probe for a remote audio URL's size + FLAC format.

use std::time::Duration;

use super::errors::describe_reqwest_error;
use super::RemoteStreamInfo;

/// HEAD for content-length, then a small `Range: bytes=0-65535` GET to (a)
/// measure throughput and (b) parse the FLAC `STREAMINFO` block for the real
/// sample rate / channels / bit depth. Never defaults silently for FLAC (a
/// wrong sample rate would silently resample hi-res).
pub async fn probe_remote_stream_info(url: &str) -> Result<RemoteStreamInfo, String> {
    use std::time::Instant;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|err| format!("create stream probe client: {err}"))?;

    let head_response = client
        .head(url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|err| format!("probe HEAD request failed: {}", describe_reqwest_error(&err)))?;

    if !head_response.status().is_success() {
        return Err(format!(
            "probe HEAD request failed with status {}",
            head_response.status()
        ));
    }

    let content_length = head_response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| "probe missing content-length header".to_string())?;

    let start_time = Instant::now();
    let range_response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .header("Range", "bytes=0-65535")
        .send()
        .await
        .map_err(|err| format!("probe range request failed: {}", describe_reqwest_error(&err)))?;

    if !range_response.status().is_success() {
        return Err(format!(
            "probe range request failed with status {}",
            range_response.status()
        ));
    }

    let initial_bytes = range_response
        .bytes()
        .await
        .map_err(|err| format!("read probe bytes failed: {}", describe_reqwest_error(&err)))?;

    let elapsed = start_time.elapsed();
    let speed_mbps = if elapsed.as_secs_f64() > 0.0 {
        (initial_bytes.len() as f64 / elapsed.as_secs_f64()) / (1024.0 * 1024.0)
    } else {
        10.0
    };

    // STREAMINFO parse via the shared prober (hoisted to qbz-models for the
    // cast path, #638 fix 1). The prober never guesses — the CD-shaped
    // defaults for a non-FLAC probe stay HERE so this path's behavior is
    // byte-identical to the original inline parse.
    let (sample_rate, channels, bit_depth) = match qbz_models::probe_streaminfo(&initial_bytes) {
        Some(p) => (p.sample_rate, p.channels, p.bits_per_sample),
        None => {
            log::warn!("[remote-stream] Non-FLAC probe for remote handoff, using defaults");
            (44_100, 2, 16)
        }
    };

    Ok(RemoteStreamInfo {
        content_length,
        sample_rate,
        channels,
        bit_depth,
        speed_mbps,
    })
}

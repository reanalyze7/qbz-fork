/// Compute the MD5 request signature for Qobuz CMAF API calls.
///
/// Concatenates method + sorted key-value pairs + timestamp + seed,
/// then returns the lowercase hex MD5 digest.
/// `seed` is the app seed provided by the caller.
pub fn compute_request_sig(
    method: &str,
    args: &std::collections::BTreeMap<&str, String>,
    timestamp: &str,
    seed: &str,
) -> String {
    use md5::{Digest, Md5};

    let mut hasher = Md5::new();
    hasher.update(method.as_bytes());
    for (k, v) in args {
        hasher.update(k.as_bytes());
        hasher.update(v.as_bytes());
    }
    hasher.update(timestamp.as_bytes());
    hasher.update(seed.as_bytes());

    format!("{:x}", hasher.finalize())
}

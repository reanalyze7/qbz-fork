//! Cache size-limit edit.

use crate::AppWindow;

use super::rebuild::rebuild;
use super::GB;

/// Set the cache size limit (GB), persist it to disk, and refresh.
pub fn set_limit(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle, gb: i32) {
    handle.spawn(async move {
        let bytes = (gb.max(1) as u64) * GB;
        if let Some(off) = crate::offline::get().await {
            *off.limit_bytes.lock().await = Some(bytes);
        }
        crate::offline::persist_limit(bytes).await;
        rebuild(weak).await;
    });
}

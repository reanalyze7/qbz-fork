//! PURE resolution (no Tauri, no events) of a v2 CMAF-cached row into
//! ready-to-play FLAC bytes.

use std::path::Path;

use crate::cmaf_store::{self, BundleLayout};
use crate::db::CmafBundleRow;

use super::decrypt::unwrap_and_decrypt;

/// Decrypt a v2 CMAF bundle row into plain FLAC bytes ready for
/// `player.play_data`. Returns `None` on any failure (missing init,
/// wrong-size unwrapped key, corrupt manifest, decrypt error). The
/// caller should treat `None` as a cache miss — continue to the next
/// tier or the network.
///
/// `offline_root_path` is only used to locate the secret vault's
/// install UUID file; it must match the path used at download time.
/// Passing `OfflineCacheState::get_cache_path()` is correct.
pub fn load_cmaf_bundle(
    track_id: u64,
    row: &CmafBundleRow,
    offline_root_path: &Path,
) -> Option<Vec<u8>> {
    if row.cache_format != 2 {
        return None;
    }

    let init_path = row.init_path.as_ref().or_else(|| {
        log::warn!(
            "[OfflineCache/Play] Track {} cache_format=2 but init_path is null",
            track_id
        );
        None
    })?;
    let content_key_wrapped = row.content_key_wrapped.as_ref().or_else(|| {
        log::warn!(
            "[OfflineCache/Play] Track {} cache_format=2 but content_key_wrapped is null",
            track_id
        );
        None
    })?;

    let segments_path = std::path::PathBuf::from(&row.segments_path);
    let track_dir = segments_path.parent()?.to_path_buf();
    let layout = BundleLayout {
        track_dir,
        init_path: std::path::PathBuf::from(init_path),
        segments_path: segments_path.clone(),
        manifest_path: segments_path.with_file_name("manifest.json"),
    };

    let loaded = match cmaf_store::read_bundle(&layout) {
        Ok(lb) => lb,
        Err(e) => {
            log::warn!(
                "[OfflineCache/Play] Track {} failed to read CMAF bundle: {}",
                track_id,
                e
            );
            return None;
        }
    };

    unwrap_and_decrypt(track_id, &loaded, content_key_wrapped, offline_root_path)
}

#[cfg(test)]
mod tests {
    use super::load_cmaf_bundle;
    use crate::db::CmafBundleRow;
    use std::path::Path;

    fn row_with_format(cache_format: u8) -> CmafBundleRow {
        CmafBundleRow {
            cache_format,
            segments_path: "/tmp/does-not-matter/segments.bin".to_string(),
            init_path: None,
            content_key_wrapped: None,
            infos_wrapped: None,
            format_id: None,
            n_segments: None,
        }
    }

    #[test]
    fn returns_none_for_non_v2_cache_format() {
        let row = row_with_format(1);
        assert!(load_cmaf_bundle(1, &row, Path::new("/tmp")).is_none());
    }

    #[test]
    fn returns_none_when_init_path_missing() {
        let row = row_with_format(2);
        assert!(load_cmaf_bundle(1, &row, Path::new("/tmp")).is_none());
    }
}

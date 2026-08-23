use std::collections::{HashMap, HashSet};

/// Local rows: resolve library rows by file path (blocking). Paths the
/// index doesn't know are stat'ed on the same worker — an existing file
/// renders as a filename-fallback row instead of hiding (D11 nuance).
pub(super) async fn resolve_local(
    local_paths: Vec<String>,
) -> (HashMap<String, qbz_library::LocalTrack>, HashSet<String>) {
    if local_paths.is_empty() {
        return Default::default();
    }
    tokio::task::spawn_blocking(move || {
        let resolved = crate::library_db::with_db(|db| {
            let mut out = HashMap::new();
            for path in &local_paths {
                if let Some(track) = db.get_track_by_path(path)? {
                    out.insert(path.clone(), track);
                }
            }
            Ok(out)
        })
        .unwrap_or_default();
        let on_disk: HashSet<String> = local_paths
            .iter()
            .filter(|p| !resolved.contains_key(*p))
            .filter(|p| std::path::Path::new(p.as_str()).exists())
            .cloned()
            .collect();
        (resolved, on_disk)
    })
    .await
    .unwrap_or_default()
}

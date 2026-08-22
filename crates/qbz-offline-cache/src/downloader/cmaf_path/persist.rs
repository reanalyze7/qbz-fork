//! Step 2: wrap the bundle's keying material via the secret vault, persist
//! the encrypted bundle to disk, and flip the DB row to v2.

use qbz_qobuz::cmaf::CmafRawBundle;

use crate::cmaf_store::BundleLayout;

/// Wraps the content key + infos, persists the bundle to disk, and records
/// the v2 fields (+ `mark_complete`) on the DB row. Returns the on-disk
/// layout and total persisted size.
pub(super) async fn wrap_persist_and_record(
    track_id: u64,
    db: &std::sync::Arc<tokio::sync::Mutex<Option<crate::OfflineCacheDb>>>,
    offline_root_path: &std::path::Path,
    bundle: &CmafRawBundle,
) -> Result<(BundleLayout, u64), String> {
    // Open the secret vault and wrap the keying material on a BLOCKING thread.
    // The OS keyring (secret-service) does a synchronous D-Bus round-trip via
    // zbus, which PANICS ("cannot start a runtime from within a runtime") when
    // run on an async worker. spawn_blocking moves it off the async pool — the
    // same reason the playback decrypt path already uses spawn_blocking.
    let offline_root_for_vault = offline_root_path.to_path_buf();
    let content_key = bundle.content_key;
    let infos = bundle.infos.clone();
    let (content_key_wrapped, infos_wrapped) = tokio::task::spawn_blocking(move || {
        let vault = crate::secret_vault::get_or_init(&offline_root_for_vault)
            .map_err(|e| format!("SecretBox init failed: {}", e))?;
        let ck = vault
            .wrap(&content_key)
            .map_err(|e| format!("Failed to wrap content_key: {}", e))?;
        let inf = vault
            .wrap(infos.as_bytes())
            .map_err(|e| format!("Failed to wrap infos: {}", e))?;
        Ok::<(Vec<u8>, Vec<u8>), String>((ck, inf))
    })
    .await
    .map_err(|e| format!("vault task join failed: {}", e))??;

    // Persist the encrypted bundle to disk.
    let (layout, total_bytes) =
        crate::cmaf_store::persist_bundle(offline_root_path, track_id, bundle)?;

    // Flip the DB row to v2 and store the wrapped keying material.
    {
        let db_guard = db.lock().await;
        let db_ref = db_guard
            .as_ref()
            .ok_or_else(|| "Offline cache DB not open".to_string())?;
        db_ref.set_cmaf_bundle(
            track_id,
            layout.segments_path.to_string_lossy().as_ref(),
            layout.init_path.to_string_lossy().as_ref(),
            &content_key_wrapped,
            &infos_wrapped,
            bundle.format_id,
            bundle.n_segments as u32,
            total_bytes,
        )?;
        db_ref
            .mark_complete(track_id, total_bytes)
            .map_err(|e| format!("Failed to mark_complete: {}", e))?;
    }

    Ok((layout, total_bytes))
}

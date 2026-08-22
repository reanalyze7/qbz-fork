//! Public batch driver: iterates legacy track ids, migrates each, updates
//! `MigrationStatus`, and deletes the legacy file on success.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use qbz_library::LibraryDatabase;
use qbz_qobuz::QobuzClient;

use super::single_track::migrate_single_track;
use super::{MigrationError, MigrationStatus};

/// Migrate all legacy cached files
pub async fn migrate_legacy_cached_files(
    track_ids: Vec<u64>,
    tracks_dir: PathBuf,
    offline_root: String,
    qobuz_client: Arc<RwLock<Option<QobuzClient>>>,
    library_db: Arc<Mutex<Option<LibraryDatabase>>>,
) -> MigrationStatus {
    let total = track_ids.len();
    let mut status = MigrationStatus {
        has_legacy_files: true,
        total_tracks: total,
        in_progress: true,
        ..Default::default()
    };

    for track_id in track_ids {
        let legacy_path = tracks_dir.join(format!("{}.flac", track_id));

        if !legacy_path.exists() {
            log::warn!("Legacy file not found: {}", legacy_path.display());
            status.processed += 1;
            continue;
        }

        // Lock client for this migration
        let client_guard = qobuz_client.read().await;
        let Some(client) = client_guard.as_ref() else {
            status.failed += 1;
            status.errors.push(MigrationError {
                track_id,
                error_message: "QobuzClient not initialized".to_string(),
            });
            status.processed += 1;
            continue;
        };

        match migrate_single_track(
            track_id,
            legacy_path.clone(),
            &offline_root,
            client,
            library_db.clone(),
        )
        .await
        {
            Ok(_) => {
                status.successful += 1;

                // Delete legacy file after successful migration
                if let Err(e) = std::fs::remove_file(&legacy_path) {
                    log::warn!(
                        "Failed to delete legacy file {}: {}",
                        legacy_path.display(),
                        e
                    );
                }
            }
            Err(e) => {
                status.failed += 1;
                status.errors.push(MigrationError {
                    track_id,
                    error_message: e,
                });
                log::error!(
                    "Failed to migrate track {}: {}",
                    track_id,
                    status.errors.last().unwrap().error_message
                );
            }
        }

        drop(client_guard);

        status.processed += 1;
    }

    status.in_progress = false;
    status.completed = true;

    log::info!(
        "Migration complete: {}/{} successful, {} failed",
        status.successful,
        total,
        status.failed
    );

    status
}

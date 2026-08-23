//! Load a collection off the UI thread.

use qbz_models::mixtape::MixtapeCollection;

/// Load a collection (items hydrated) off the UI/event-loop thread, on a
/// blocking worker, reusing the detail module's read path. Returns `None` when
/// the DB is unavailable or the id is unknown.
pub(crate) async fn load_collection(collection_id: &str) -> Option<MixtapeCollection> {
    let id = collection_id.to_string();
    tokio::task::spawn_blocking(move || crate::myqbz_detail::get_collection(&id))
        .await
        .ok()
        .flatten()
}

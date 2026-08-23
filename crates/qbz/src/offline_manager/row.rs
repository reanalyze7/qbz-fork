//! Worker-built row data, converted to `OfflineRow` on the UI thread.

/// Worker-built, `Send` row data. Converted to the (non-`Send`) `OfflineRow`
/// on the UI thread; the cover is pre-decoded to size on the worker
/// (`DecodedPixels` is `Send`, `slint::Image` is not).
pub(super) struct RowData {
    pub kind: &'static str,
    pub album_id: String,
    pub track_id: String,
    pub title: String,
    pub subtitle: String,
    pub meta: String,
    pub status: i32,
    pub progress: f32,
    pub cover: Option<crate::artwork::DecodedPixels>,
    pub number: String,
}

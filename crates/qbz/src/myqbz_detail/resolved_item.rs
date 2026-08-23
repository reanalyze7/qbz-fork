//! The live-resolved per-row display values cached by `RESOLVE_CACHE`.

/// Values the `resolve_items` pass derives per row, keyed
/// `source|source_item_id` in `RESOLVE_CACHE`, and re-hydrated by
/// `model::to_item` on every filter/sort/search re-derive.
#[derive(Clone, Default)]
pub(super) struct ResolvedItem {
    /// "qobuz" | "local".
    pub(super) source_kind: String,
    /// "hires" | "cd" | "" — the row's `QualityBadgeFull` tier.
    pub(super) quality_tier: String,
    /// "24-bit / 96 kHz" etc; "" when tier is "".
    pub(super) quality_detail: String,
    /// Uppercased TYPE-column label (ALBUM / EP / SINGLE / TRACK / PLAYLIST).
    pub(super) type_label: String,
    /// First resolved track's artwork (bare local file path / Qobuz URL —
    /// the `file://` prefix is stripped). Backfills rows whose stored
    /// `artwork_url` was empty (e.g. disco-builder local items saved with
    /// NULL art before the builder carried the cover).
    pub(super) artwork_url: String,
    /// First resolved track's numeric Qobuz artist id ("" when the track
    /// carries none — local items). The stored `MixtapeCollectionItem`
    /// only has the artist NAME (subtitle), so this is what lets a Qobuz
    /// item's artist link open the QOBUZ artist page instead of falling back
    /// to the LocalLibrary Artists tab.
    pub(super) artist_id: String,
}

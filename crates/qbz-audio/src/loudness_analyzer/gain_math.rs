use super::MAX_GAIN_DB;
use crate::loudness::db_to_linear;

/// Convert a dB adjustment to a capped linear gain factor.
pub(super) fn compute_gain_capped(adjustment_db: f32) -> f32 {
    let capped_db = adjustment_db.min(MAX_GAIN_DB);
    db_to_linear(capped_db)
}

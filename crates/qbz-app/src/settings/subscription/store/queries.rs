use super::SubscriptionStateStore;
use crate::settings::subscription::state::SubscriptionState;
use crate::settings::subscription::GRACE_PERIOD_SECS;
use rusqlite::params;

impl SubscriptionStateStore {
    pub fn get_state(&self) -> Result<SubscriptionState, String> {
        self.conn
            .query_row(
                "SELECT invalid_since, last_invalid_at, last_valid_at, last_checked_at, downloads_purged_at
                 FROM subscription_state WHERE id = 1",
                [],
                |row| {
                    Ok(SubscriptionState {
                        invalid_since: row.get(0)?,
                        last_invalid_at: row.get(1)?,
                        last_valid_at: row.get(2)?,
                        last_checked_at: row.get(3)?,
                        downloads_purged_at: row.get(4)?,
                    })
                },
            )
            .map_err(|e| format!("Failed to read subscription state: {}", e))
    }

    pub fn mark_valid(&self, now: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE subscription_state
                 SET invalid_since = NULL,
                     last_valid_at = ?1,
                     last_checked_at = ?1
                 WHERE id = 1",
                params![now],
            )
            .map_err(|e| format!("Failed to update subscription state: {}", e))?;
        Ok(())
    }

    pub fn mark_invalid(&self, now: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE subscription_state
                 SET invalid_since = COALESCE(invalid_since, ?1),
                     last_invalid_at = ?1,
                     last_checked_at = ?1
                 WHERE id = 1",
                params![now],
            )
            .map_err(|e| format!("Failed to update subscription state: {}", e))?;
        Ok(())
    }

    pub fn mark_offline_cache_purged(&self, now: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE subscription_state SET downloads_purged_at = ?1 WHERE id = 1",
                params![now],
            )
            .map_err(|e| format!("Failed to update purge timestamp: {}", e))?;
        Ok(())
    }

    pub fn should_purge_offline_cache(&self, now: i64) -> Result<bool, String> {
        let state = self.get_state()?;
        let Some(invalid_since) = state.invalid_since else {
            return Ok(false);
        };
        if now - invalid_since < GRACE_PERIOD_SECS {
            return Ok(false);
        }
        if let Some(purged_at) = state.downloads_purged_at {
            if purged_at >= invalid_since {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// D4 playback gate: may the offline cache serve FULL tracks right now?
    ///
    /// Binary by design — within the grace window the cache plays complete
    /// tracks; past it, playback is refused outright. There is NO degraded
    /// 30-second-preview path (the owner's explicit requirement: Qobuz's
    /// preview behavior must never appear here).
    ///
    /// `invalid_since == None` (never observed invalid, including the
    /// never-checked default) ⇒ allowed: the grace clock only starts when a
    /// login verdict explicitly reports the account ineligible.
    pub fn offline_playback_allowed(&self, now: i64) -> Result<bool, String> {
        let state = self.get_state()?;
        let Some(invalid_since) = state.invalid_since else {
            return Ok(true);
        };
        Ok(now - invalid_since < GRACE_PERIOD_SECS)
    }
}

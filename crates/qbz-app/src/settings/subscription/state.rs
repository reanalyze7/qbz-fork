use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionState {
    pub invalid_since: Option<i64>,
    pub last_invalid_at: Option<i64>,
    pub last_valid_at: Option<i64>,
    pub last_checked_at: Option<i64>,
    pub downloads_purged_at: Option<i64>,
}

impl Default for SubscriptionState {
    fn default() -> Self {
        Self {
            invalid_since: None,
            last_invalid_at: None,
            last_valid_at: None,
            last_checked_at: None,
            downloads_purged_at: None,
        }
    }
}

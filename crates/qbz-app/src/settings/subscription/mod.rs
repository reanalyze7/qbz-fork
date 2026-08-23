//! Subscription validity tracking for offline download compliance.
//!
//! Tracks when a user was first observed without a valid subscription. If the
//! invalid state persists for more than the grace period, offline downloads are
//! purged by the host-side session lifecycle.

mod handle;
mod state;
mod store;
#[cfg(test)]
mod tests;

/// How long we keep honoring offline access after the server first reports an
/// invalid subscription.
///
/// Qobuz's own mobile app gives a 30-day offline grace window, so QBZ matches
/// that posture for compliance. Shorter windows would punish users on flaky
/// networks; longer windows would be more lenient than the official client.
///
/// The primary protection for offline files is the CMAF-at-rest cache format.
/// This grace period is an additional compliance guard, not the main defense.
const GRACE_PERIOD_SECS: i64 = 30 * 24 * 60 * 60;

pub use handle::{create_empty_subscription_state, create_subscription_state, SubscriptionStateState};
pub use state::SubscriptionState;
pub use store::SubscriptionStateStore;

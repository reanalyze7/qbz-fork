use super::store::SubscriptionStateStore;
use std::sync::{Arc, Mutex};

pub type SubscriptionStateState = Arc<Mutex<Option<SubscriptionStateStore>>>;

pub fn create_subscription_state() -> Result<SubscriptionStateState, String> {
    let store = SubscriptionStateStore::new()?;
    Ok(Arc::new(Mutex::new(Some(store))))
}

pub fn create_empty_subscription_state() -> SubscriptionStateState {
    Arc::new(Mutex::new(None))
}

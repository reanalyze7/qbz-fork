mod ops;
mod state;
mod store;

use serde::{Deserialize, Serialize};

pub use state::AllowedOriginsState;
pub use store::AllowedOriginsStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedOrigin {
    pub id: i64,
    pub origin: String,
    pub is_default: bool,
}

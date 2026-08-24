//! Full-load the metadata-grouped Albums set (the Albums tab loads the
//! entire set in one shot; search/sort/filter/group are all derived
//! client-side over the cached set — see `super::derive`).

mod reload;
mod seed;
mod spawn;
mod state;

pub use reload::*;
pub use seed::*;
pub use state::*;

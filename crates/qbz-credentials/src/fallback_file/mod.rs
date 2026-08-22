//! The encrypted fallback file: the authoritative on-disk credential store,
//! plus migration from the pre-encryption legacy XOR format.

mod legacy;
mod load;
mod store;

pub(crate) use load::load_from_fallback;
pub(crate) use store::{clear_fallback, has_fallback_credentials, save_to_fallback};

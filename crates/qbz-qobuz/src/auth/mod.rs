//! Authentication and request signing

mod cmaf;
mod login;
mod signing;

pub use cmaf::{sign_file_url, sign_session_start, CMAF_SEED};
pub use login::parse_login_response;
pub use signing::{
    generate_signature, get_timestamp, sign_get_favorites, sign_get_file_url, sign_request,
    sign_search,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

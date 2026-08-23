mod albums;
mod artists;
mod shared;

use super::BlacklistService;

pub(super) fn svc() -> BlacklistService {
    BlacklistService::new_in_memory().expect("svc")
}

use qbz_core::FrontendAdapter;

use super::AppRuntime;
use crate::user_data::UserDataPaths;

impl<A: FrontendAdapter + Send + Sync + 'static> AppRuntime<A> {
    /// First real login takes possession of the guest profile (#553): data
    /// built while logged off (local library, mixtapes, per-user prefs —
    /// `users/0/`) is renamed to this account IF the account has no profile
    /// on this machine yet. An existing profile always wins — the guest
    /// dirs then stay parked at user 0, no merge is attempted. Best-effort:
    /// a failed rename logs and falls through to a fresh profile; login
    /// must never break here.
    pub(super) fn adopt_guest_profile(user_id: u64) {
        if user_id == 0 {
            return;
        }
        for (kind, guest, target) in [
            (
                "data",
                UserDataPaths::data_dir_for(0),
                UserDataPaths::data_dir_for(user_id),
            ),
            (
                "cache",
                UserDataPaths::cache_dir_for(0),
                UserDataPaths::cache_dir_for(user_id),
            ),
        ] {
            let (Ok(guest), Ok(target)) = (guest, target) else {
                continue;
            };
            if guest.is_dir() && !target.exists() {
                match std::fs::rename(&guest, &target) {
                    Ok(()) => {
                        log::info!("[AppRuntime] guest profile {kind} adopted by the new session")
                    }
                    Err(e) => log::warn!(
                        "[AppRuntime] guest profile {kind} adoption failed ({e}); starting fresh"
                    ),
                }
            }
        }
    }
}

//! Per-key keyring operations, each running behind the wall-clock timeout
//! and updating the broken-state latch in `super`.

use keyring::Entry;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::{keyring_is_broken, mark_keyring_broken, mark_keyring_working, SERVICE_NAME};

/// Per-operation wall-clock limit for any Secret Service / keyring call.
const KEYRING_OP_TIMEOUT: Duration = Duration::from_millis(2500);

/// Run a blocking keyring closure on a worker thread and return its result
/// within `KEYRING_OP_TIMEOUT`. Times out cleanly if the closure is still
/// stuck (typically because the user is looking at an unlock dialog).
fn run_with_keyring_timeout<T, F>(op_name: &'static str, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(KEYRING_OP_TIMEOUT).map_err(|_| {
        format!(
            "keyring {} did not complete within {}ms (likely blocked on a user dialog)",
            op_name,
            KEYRING_OP_TIMEOUT.as_millis()
        )
    })
}

/// Read a value from the keyring. Returns `None` if the entry does not exist
/// OR if the keyring is unavailable / broken for this session. The caller
/// must always have a file-based fallback ready.
pub(crate) fn keyring_get(key: &str) -> Option<String> {
    if keyring_is_broken() {
        return None;
    }
    let service = SERVICE_NAME.to_string();
    let key_owned = key.to_string();
    match run_with_keyring_timeout("get", move || {
        Entry::new(&service, &key_owned).and_then(|e| e.get_password())
    }) {
        Ok(Ok(value)) => {
            mark_keyring_working();
            Some(value)
        }
        Ok(Err(keyring::Error::NoEntry)) => {
            mark_keyring_working();
            None
        }
        Ok(Err(e)) => {
            mark_keyring_broken(&format!("get failed: {}", e));
            None
        }
        Err(reason) => {
            mark_keyring_broken(&reason);
            None
        }
    }
}

/// Write a value to the keyring. Returns `true` on success, `false` on any
/// failure (timeout, locked collection, no backend, etc.). The caller must
/// have already persisted the value through its authoritative path (file).
pub(crate) fn keyring_set(key: &str, value: &str) -> bool {
    if keyring_is_broken() {
        return false;
    }
    let service = SERVICE_NAME.to_string();
    let key_owned = key.to_string();
    let value_owned = value.to_string();
    match run_with_keyring_timeout("set", move || {
        Entry::new(&service, &key_owned).and_then(|e| e.set_password(&value_owned))
    }) {
        Ok(Ok(())) => {
            mark_keyring_working();
            true
        }
        Ok(Err(e)) => {
            mark_keyring_broken(&format!("set failed: {}", e));
            false
        }
        Err(reason) => {
            mark_keyring_broken(&reason);
            false
        }
    }
}

/// Delete a keyring entry. Silent no-op if the keyring is broken or the
/// entry already doesn't exist.
pub(crate) fn keyring_delete(key: &str) {
    if keyring_is_broken() {
        return;
    }
    let service = SERVICE_NAME.to_string();
    let key_owned = key.to_string();
    match run_with_keyring_timeout("delete", move || {
        Entry::new(&service, &key_owned).and_then(|e| e.delete_credential())
    }) {
        Ok(Ok(())) | Ok(Err(keyring::Error::NoEntry)) => {
            mark_keyring_working();
        }
        Ok(Err(e)) => {
            log::debug!("[Credentials] Keyring delete failed (not critical): {}", e);
        }
        Err(reason) => {
            mark_keyring_broken(&reason);
        }
    }
}

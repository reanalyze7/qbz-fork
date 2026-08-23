//! Selection mutation + persist trigger.

use super::persistence::save_persisted;
use super::STATE;

/// Toggle a genre id in the current context's selection. Returns true if
/// the selection changed (so the caller can re-fetch / re-derive).
pub fn toggle(id_str: &str) -> bool {
    let Ok(id) = id_str.parse::<u64>() else {
        return false;
    };
    let Ok(mut s) = STATE.lock() else {
        return false;
    };
    {
        let sel = s.cur_mut();
        if let Some(pos) = sel.iter().position(|x| *x == id) {
            sel.remove(pos);
        } else {
            sel.push(id);
        }
    }
    let (contexts, rem) = (s.selected.clone(), s.remember);
    drop(s);
    save_persisted(&contexts, rem);
    true
}

pub fn clear() {
    let Ok(mut s) = STATE.lock() else {
        return;
    };
    s.cur_mut().clear();
    let (contexts, rem) = (s.selected.clone(), s.remember);
    drop(s);
    save_persisted(&contexts, rem);
}

pub fn set_remember(remember: bool) {
    let Ok(mut s) = STATE.lock() else {
        return;
    };
    s.remember = remember;
    let contexts = s.selected.clone();
    drop(s);
    save_persisted(&contexts, remember);
}

// ============================ save result (§4.3) ============================

pub const SAVE_TITLE: &str = "Saved";
pub const RESULT_HINT: &str = "Enter / Esc close";

pub const SAVED_DISK_ONLY: &str =
    "saved to disk — daemon didn't answer; changes apply on restart";
pub const RELOAD_REFUSED: &str =
    "saved to disk — daemon answered but refused the reload; restart it:\n  systemctl --user restart qbzd";

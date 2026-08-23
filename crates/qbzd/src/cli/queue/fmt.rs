// ============================ shared rendering ============================

pub(super) fn fmt_mmss(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

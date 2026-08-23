use serde_json::Value;

pub(super) fn str_at(p: &Value, path: &[&str]) -> String {
    let mut cur = p;
    for k in path {
        match cur.get(k) {
            Some(v) => cur = v,
            None => return String::new(),
        }
    }
    cur.as_str().unwrap_or("").to_string()
}

pub(super) fn fmt_mmss(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub(super) fn fmt_uptime(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let mins = (secs % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

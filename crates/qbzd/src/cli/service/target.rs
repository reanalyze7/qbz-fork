use std::process::Command;

pub(super) struct Target {
    pub(super) user: String,
    pub(super) group: String,
    pub(super) uid: String,
    pub(super) home: String,
    pub(super) xdg_runtime: String,
    pub(super) bin: String,
}

pub(super) fn resolve(user: Option<String>, bin: Option<String>) -> Target {
    let bin = bin
        .or_else(|| std::env::current_exe().ok().and_then(|p| p.to_str().map(String::from)))
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "/usr/bin/qbzd".to_string());

    let user = user
        .or_else(|| std::env::var("USER").ok().filter(|u| !u.is_empty()))
        .unwrap_or_else(|| "qbz".to_string());

    let (uid, home) = passwd(&user);
    let group = id_group(&user).unwrap_or_else(|| user.clone());

    // For the CURRENT user, the live XDG_RUNTIME_DIR is authoritative (captures a
    // non-default path); for another user, derive it from the uid.
    let is_current = std::env::var("USER").ok().as_deref() == Some(user.as_str());
    let xdg_runtime = std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .filter(|s| is_current && !s.is_empty())
        .unwrap_or_else(|| format!("/run/user/{uid}"));

    Target { user, group, uid, home, xdg_runtime, bin }
}

/// `(uid, home)` for a user via `getent passwd` (name:x:uid:gid:gecos:home:sh),
/// falling back to `id -u` + a `/home/<user>` heuristic on a passwd-less box.
fn passwd(user: &str) -> (String, String) {
    if let Some(line) = run(&["getent", "passwd", user]) {
        let f: Vec<&str> = line.trim().split(':').collect();
        if f.len() >= 7 && !f[2].is_empty() {
            return (f[2].to_string(), f[5].to_string());
        }
    }
    let uid = run(&["id", "-u", user]).unwrap_or_else(|| "1000".to_string());
    (uid, format!("/home/{user}"))
}

fn id_group(user: &str) -> Option<String> {
    run(&["id", "-gn", user]).filter(|s| !s.is_empty())
}

/// Run a command and return its trimmed stdout on success, else None.
fn run(args: &[&str]) -> Option<String> {
    let out = Command::new(args[0]).args(&args[1..]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

pub(super) fn detect_init() -> Option<String> {
    let exists = |p: &str| std::path::Path::new(p).exists();
    if exists("/run/systemd/system") {
        Some("systemd".into())
    } else if exists("/run/openrc") || exists("/sbin/openrc") || exists("/etc/init.d/functions.sh") {
        Some("openrc".into())
    } else if exists("/run/runit") || exists("/etc/runit") || exists("/etc/sv") {
        Some("runit".into())
    } else {
        None
    }
}

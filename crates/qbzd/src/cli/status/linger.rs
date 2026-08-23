use crate::cli::copy;

/// `loginctl show-user $USER -p Linger` → the §1.4 linger warning on `Linger=no`.
/// Any failure (no loginctl, no session) → no warning.
pub(super) fn linger_warning() -> Option<String> {
    let user = std::env::var("USER").ok().filter(|u| !u.is_empty())?;
    let out = std::process::Command::new("loginctl")
        .args(["show-user", &user, "-p", "Linger"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    if text.trim() == "Linger=no" {
        Some(copy::linger_off(&user))
    } else {
        None
    }
}

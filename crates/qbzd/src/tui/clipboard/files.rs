use std::path::PathBuf;

/// The directory the `w` (write) action and the file-fallback tier save into —
/// ALWAYS under the operator's home, NEVER a system path (HARD RULE: the wizard
/// never writes a live config file).
pub fn wizard_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join("qbzd-wizard")
}

/// Write `text` to `~/qbzd-wizard/<stem>.conf`, creating the dir. Returns the
/// full path so the caller can print it verbatim.
pub fn write_wizard_file(stem: &str, text: &str) -> std::io::Result<PathBuf> {
    let dir = wizard_dir();
    std::fs::create_dir_all(&dir)?;
    let safe: String = stem
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    let stem = if safe.trim_matches('-').is_empty() { "dac".to_string() } else { safe };
    let path = dir.join(format!("{stem}.conf"));
    std::fs::write(&path, text)?;
    Ok(path)
}

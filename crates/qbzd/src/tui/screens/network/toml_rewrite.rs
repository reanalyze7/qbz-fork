/// Whole-file rewrite: parse `existing` (or start empty), update only
/// [server] bind/port/token, keep every other key verbatim (§3.5). An empty or
/// whitespace-only token REMOVES the key (open control plane).
pub fn rewrite_toml(
    existing: &str,
    bind: &str,
    port: u16,
    token: Option<&str>,
) -> Result<String, String> {
    let mut root: toml::Table = if existing.trim().is_empty() {
        toml::Table::new()
    } else {
        toml::from_str(existing).map_err(|e| format!("cannot parse qbzd.toml: {e}"))?
    };
    let server = root
        .entry("server".to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    let st = server
        .as_table_mut()
        .ok_or_else(|| "qbzd.toml [server] is not a table".to_string())?;
    st.insert("bind".to_string(), toml::Value::String(bind.to_string()));
    st.insert("port".to_string(), toml::Value::Integer(port as i64));
    match token {
        Some(t) if !t.trim().is_empty() => {
            st.insert("token".to_string(), toml::Value::String(t.to_string()));
        }
        _ => {
            st.remove("token");
        }
    }
    toml::to_string(&root).map_err(|e| e.to_string())
}

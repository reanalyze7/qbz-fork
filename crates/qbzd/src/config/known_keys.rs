/// Known keys, one entry per (table, key). Kept literal so the sweep and the
/// spec table diff cleanly. Released keys are never renamed without an alias.
const KNOWN: &[(&str, &str)] = &[
    ("", "config_version"),
    ("", "data_root"),
    ("server", "bind"),
    ("server", "port"),
    ("server", "token"),
    ("log", "level"),
    ("mpris", "enabled"),
];

pub(super) fn sweep(v: &toml::Value, table: &str, warns: &mut Vec<String>) {
    if let toml::Value::Table(map) = v {
        for (k, inner) in map {
            match inner {
                toml::Value::Table(_) if table.is_empty() => sweep(inner, k, warns),
                _ if !KNOWN.contains(&(table, k.as_str())) => {
                    warns.push(if table.is_empty() {
                        k.clone()
                    } else {
                        format!("[{table}].{k}")
                    });
                }
                _ => {}
            }
        }
    }
}

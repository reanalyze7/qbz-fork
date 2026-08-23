use super::QbzdConfig;
use super::known_keys::sweep;

impl QbzdConfig {
    pub fn from_str(text: &str) -> Result<(Self, Vec<String>), String> {
        let value: toml::Value = toml::from_str(text).map_err(|e| e.to_string())?;
        let mut warns = Vec::new();
        sweep(&value, "", &mut warns);
        let cfg: QbzdConfig = value.try_into().map_err(|e: toml::de::Error| e.to_string())?;
        Ok((cfg, warns))
    }
    pub fn load(path: &std::path::Path) -> Result<(Self, Vec<String>), String> {
        match std::fs::read_to_string(path) {
            Ok(t) => Self::from_str(&t),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok((Self::default(), Vec::new()))
            }
            Err(e) => Err(format!("cannot read {}: {e}", path.display())),
        }
    }
}

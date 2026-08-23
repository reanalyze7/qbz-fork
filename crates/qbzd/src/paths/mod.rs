// crates/qbzd/src/paths/ — daemon profile roots (01-architecture.md §4.2).
// The daemon owns a fully separate profile from the desktop: `~/.config/qbzd`,
// `~/.local/share/qbzd`, `~/.cache/qbzd`. It NEVER opens desktop
// `~/.local/share/qbz/**` at runtime (that only happens inside
// `settings export --from desktop`, 04 §4.1 — out of scope here).
mod permissions;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use permissions::ensure_config_dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileRoots {
    pub config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
}

impl ProfileRoots {
    /// Resolve the three profile roots.
    ///
    /// - `config_override`: the `--config <path>` argument (a FILE path); its
    ///   parent directory becomes the config root. `None` falls back to
    ///   `dirs::config_dir()/qbzd`.
    /// - `data_root_override`: the already-parsed `qbzd.toml` `data_root`
    ///   value (a container override, e.g. for a Pi SD-card layout). `None`
    ///   falls back to `dirs::data_dir()/qbzd`.
    ///
    /// Cache is `dirs::cache_dir()/qbzd` UNLESS `data_root` was overridden,
    /// in which case cache = `<data_root>/cache` — never
    /// `<data_root>/../qbzd-cache`, which would walk outside the container.
    ///
    /// The config directory is created (mode 0700 on unix) on first use;
    /// data/cache directories are created by their respective owners
    /// (the instance lock creates the data root, cache writers create theirs).
    pub fn resolve(config_override: Option<&Path>, data_root_override: Option<&Path>) -> Self {
        let config = match config_override {
            Some(config_file) => config_file
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(".")),
            None => default_config_dir(),
        };
        let data = match data_root_override {
            Some(dir) => dir.to_path_buf(),
            None => default_data_dir(),
        };
        let cache = match data_root_override {
            Some(_) => data.join("cache"),
            None => default_cache_dir(),
        };

        ensure_config_dir(&config);

        Self { config, data, cache }
    }
}

fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qbzd")
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qbzd")
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qbzd")
}

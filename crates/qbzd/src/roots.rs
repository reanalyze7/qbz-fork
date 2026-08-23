use crate::{config, paths};

/// Resolve the daemon profile roots for a local CLI auth operation. `login` and
/// `logout` write the credential file into the config root and nudge the LOCAL
/// daemon, so — like `run` — they honor a `qbzd.toml` `data_root` override while
/// keeping the config root at its XDG default.
pub fn login_roots() -> paths::ProfileRoots {
    let bootstrap = paths::ProfileRoots::resolve(None, None);
    let cfg_path = bootstrap.config.join("qbzd.toml");
    let data_root = config::QbzdConfig::load(&cfg_path)
        .ok()
        .and_then(|(c, _)| c.data_root);
    paths::ProfileRoots::resolve(None, data_root.as_deref().map(std::path::Path::new))
}

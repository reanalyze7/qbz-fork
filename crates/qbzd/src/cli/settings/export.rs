// crates/qbzd/src/cli/settings/export.rs — `qbzd settings export` (T12,
// 04-settings-portability.md §4.1).

use std::path::PathBuf;

use qbz_app::settings::bundle::{self, ExportOptions, ExportSource, ProfilePaths};

use crate::paths::ProfileRoots;

/// `qbzd settings export [FILE] [--from daemon|desktop] [--include-auth]` (⬇,
/// 04-settings-portability.md §4.1). Reads the daemon (default) or the desktop's
/// GLOBAL stores, writes ONE versioned JSON bundle at 0600. Exit: 0 · 1 · 2.
pub fn export(roots: &ProfileRoots, file: Option<String>, from: &str, include_auth: bool) -> i32 {
    let source = match from {
        "daemon" => ExportSource::Daemon(ProfilePaths {
            config_root: roots.config.clone(),
            data_root: roots.data.clone(),
        }),
        "desktop" => ExportSource::Desktop,
        other => {
            eprintln!("error: invalid --from '{other}' — expected 'daemon' or 'desktop'");
            return 2;
        }
    };

    let bundle = match bundle::export(source, &ExportOptions { include_auth }) {
        Ok(b) => b,
        Err(bundle::BundleError::NoDesktopProfile) => {
            eprintln!("{}", crate::cli::copy::bundle_no_desktop_profile());
            return 1;
        }
        Err(bundle::BundleError::TokenDecryptFailed) => {
            eprintln!("{}", crate::cli::copy::bundle_token_decrypt_failed());
            return 1;
        }
        Err(e) => {
            eprintln!("error: {e}");
            return 1;
        }
    };

    let path = file.unwrap_or_else(bundle::default_filename);
    if let Err(e) = bundle::write_bundle_file(&PathBuf::from(&path), &bundle) {
        eprintln!("error: {e}");
        return 1;
    }

    // The §3 warning prints whenever ANY secret actually made it into the file:
    // the auth token OR a non-blank scrobbler secret (--include-auth exports
    // scrobbler tokens even when the Qobuz token itself is absent).
    if bundle.contains_secrets() {
        println!("{}", crate::cli::copy::bundle_secret_warning(&path));
    } else {
        println!("{}", crate::cli::copy::bundle_export_success(&path));
    }
    0
}

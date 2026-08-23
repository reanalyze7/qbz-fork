/// Everything that can go wrong in the engine. `Display` renders a plain
/// message; the CLI maps the daemon-facing variants to the verbatim 04 copies
/// in `cli::copy`.
#[derive(Debug)]
pub enum BundleError {
    /// Step 1: unreadable file / invalid JSON.
    Parse(String),
    /// Step 2: `schema_version` missing or non-integer.
    VersionMalformed,
    /// Step 2: bundle newer than this importer (§5.6).
    VersionTooNew { bundle: i64, supported: i64 },
    /// Export: no desktop profile found (§4.1).
    NoDesktopProfile,
    /// Export: `--include-auth` but the desktop token would not decrypt (IV1,
    /// §4.1 — portal-secret bound to the desktop session).
    TokenDecryptFailed,
    /// Any store/file I/O failure (export write or apply write).
    Io(String),
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::Parse(m) => write!(f, "cannot read bundle: {m}"),
            BundleError::VersionMalformed => {
                write!(f, "bundle has no valid integer schema_version")
            }
            BundleError::VersionTooNew { bundle, supported } => write!(
                f,
                "this bundle is schema v{bundle}; this qbzd understands up to v{supported}"
            ),
            BundleError::NoDesktopProfile => write!(f, "no desktop profile found"),
            BundleError::TokenDecryptFailed => {
                write!(f, "could not decrypt the desktop Qobuz token")
            }
            BundleError::Io(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for BundleError {}

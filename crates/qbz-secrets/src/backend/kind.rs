/// Which backend is active at runtime. Useful for diagnostics / UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    /// Master key lives in the OS keyring. Moving the offline cache to
    /// another machine makes it unreadable. Gold-standard device binding.
    Keyring,
    /// Master key is derived on the fly from `machine-id` + a persistent
    /// per-install UUID. Still device-bound, but reversible by anyone
    /// with filesystem access to both sources. Used when the OS keyring
    /// is unavailable (headless daemon, Pi-like setups).
    KdfFallback,
}

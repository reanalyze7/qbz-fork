use crate::config::QbzdConfig;
use crate::lock::LockError;

/// Resolve `[server] bind:port` to a `SocketAddr` (01 §10.1). A malformed value
/// is a fatal boot error that names the fix (exit 1).
pub(super) fn resolve_bind_addr(cfg: &QbzdConfig) -> Result<std::net::SocketAddr, String> {
    use std::net::ToSocketAddrs;
    let hostport = format!("{}:{}", cfg.server.bind, cfg.server.port);
    hostport
        .to_socket_addrs()
        .map_err(|e| {
            format!("error: invalid [server] bind/port '{hostport}': {e}\n  → set a valid ip and port in ~/.config/qbzd/qbzd.toml")
        })?
        .next()
        .ok_or_else(|| {
            format!("error: [server] '{hostport}' resolved to no address\n  → set a valid ip and port in ~/.config/qbzd/qbzd.toml")
        })
}

/// The step-5 bind-conflict diagnosis (02 §8.1-5 / §2.2): a qbzd occupant on a
/// different data root vs. an unrelated process on the port.
pub(crate) fn diagnose_port_conflict(addr: std::net::SocketAddr) -> String {
    if crate::api::probe_is_qbzd(addr) {
        crate::cli::copy::foreign_qbzd(&addr.to_string())
    } else {
        crate::cli::copy::port_in_use(addr.port())
    }
}

/// Render an [`InstanceLock`] failure. For the already-running case this prints
/// the frozen exit-3 error voice (02 §1.3/§1.4) and exits 3 directly — the new
/// process must never clobber the running one. An I/O failure returns a String
/// that propagates to a generic exit 1.
pub(crate) fn diagnose_lock(e: LockError) -> String {
    match e {
        LockError::AlreadyRunning(pid) => {
            let who = pid
                .map(|p| format!("(pid {p})"))
                .unwrap_or_else(|| "(pid unknown)".to_string());
            eprintln!("error: qbzd is already running {who}");
            eprintln!("  → stop it first:  systemctl --user stop qbzd");
            eprintln!("  → or inspect it:  systemctl --user status qbzd");
            std::process::exit(3);
        }
        LockError::Io(msg) => {
            format!("error: could not take the instance lock: {msg}\n  → check permissions on the data root")
        }
    }
}

/// Park until SIGTERM or SIGINT. A second signal after this returns lets the
/// default handler take over → immediate exit (§8.2).
pub(crate) async fn wait_for_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        match (
            signal(SignalKind::terminate()),
            signal(SignalKind::interrupt()),
        ) {
            (Ok(mut term), Ok(mut int)) => {
                tokio::select! {
                    _ = term.recv() => log::info!("SIGTERM received — shutting down"),
                    _ = int.recv()  => log::info!("SIGINT received — shutting down"),
                }
            }
            _ => {
                // Fall back to Ctrl-C if the SIGTERM handler could not install.
                let _ = tokio::signal::ctrl_c().await;
                log::info!("Ctrl-C received — shutting down");
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
        log::info!("Ctrl-C received — shutting down");
    }
}

use tokio::sync::watch;

use super::loop_body;
use super::probe::PROBE_TIMEOUT;
use super::types::ConnectivitySnapshot;

/// Handle to the running actor: subscribe to state, poke a recheck.
pub struct ConnectivityActor {
    rx: watch::Receiver<ConnectivitySnapshot>,
    recheck: tokio::sync::mpsc::Sender<()>,
}

impl ConnectivityActor {
    /// Spawn the actor loop on the current tokio runtime.
    pub fn spawn() -> Self {
        let (tx, rx) = watch::channel(ConnectivitySnapshot::default());
        let (recheck_tx, recheck_rx) = tokio::sync::mpsc::channel::<()>(4);

        tokio::spawn(async move {
            let client = match reqwest::Client::builder()
                .timeout(PROBE_TIMEOUT)
                .redirect(reqwest::redirect::Policy::none())
                .build()
            {
                Ok(c) => c,
                Err(e) => {
                    log::error!("[Connectivity] probe client build failed: {}", e);
                    return;
                }
            };

            loop_body::run(client, tx, recheck_rx).await;
        });

        Self {
            rx,
            recheck: recheck_tx,
        }
    }

    /// Subscribe to connectivity state changes.
    pub fn subscribe(&self) -> watch::Receiver<ConnectivitySnapshot> {
        self.rx.clone()
    }

    /// Current snapshot without subscribing.
    pub fn snapshot(&self) -> ConnectivitySnapshot {
        *self.rx.borrow()
    }

    /// Force an immediate re-evaluation (Settings "Check now", resume hooks,
    /// mode changes). Also clears any failing streak first.
    pub fn request_recheck(&self) {
        let _ = self.recheck.try_send(());
    }
}

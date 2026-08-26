//! Pre-analyse de loudness hors-ligne : mesurer un morceau AVANT qu'il joue.
//!
//! # Pourquoi
//!
//! L'analyseur temps reel ne peut mesurer qu'a partir du son deja joue : il
//! lui faut plusieurs secondes de morceau avant de savoir a quel volume le
//! mettre. Resultat, un titre inconnu demarrait au volume brut puis se
//! corrigeait en cours de route — l'ecart s'entendait a chaque transition.
//!
//! Or les octets du titre suivant sont en memoire bien avant qu'il commence
//! (prefetch gapless ~10 s a l'avance, ou promotion "full track buffered" du
//! chemin streaming). Ce thread les decode et les mesure pendant ce temps, et
//! remplit le cache : quand le morceau prend l'antenne, son gain est deja
//! connu et pose des la premiere note.
//!
//! Le decodage vit ici et non dans `qbz-audio` parce que `decode_with_fallback`
//! est le seul decodeur qui gere le CMAF/mp4 de Qobuz.

mod job;
mod worker;

pub(crate) use job::OfflineJob;

use std::sync::mpsc::{self, SyncSender};
use std::sync::Arc;
use std::thread;

use qbz_audio::LoudnessCache;

/// Poignee vers le thread de pre-analyse.
#[derive(Clone)]
pub(crate) struct OfflineLoudness {
    tx: SyncSender<OfflineJob>,
}

impl OfflineLoudness {
    pub(crate) fn spawn(cache: Arc<LoudnessCache>) -> Self {
        let (tx, rx) = mpsc::sync_channel::<OfflineJob>(4);
        thread::Builder::new()
            .name("loudness-offline".into())
            .spawn(move || {
                log::info!("[OfflineLoudness] Thread demarre");
                worker::run(rx, cache);
                log::info!("[OfflineLoudness] Thread termine");
            })
            .expect("Failed to spawn offline loudness thread");
        Self { tx }
    }

    /// Depose un travail. Ne bloque jamais : si la file est pleine, le morceau
    /// sera simplement mesure en temps reel comme avant.
    pub(crate) fn submit(&self, job: OfflineJob) {
        let track_id = job.track_id;
        if self.tx.try_send(job).is_err() {
            log::debug!(
                "[OfflineLoudness] File pleine, pre-analyse ignoree pour la piste {}",
                track_id
            );
        }
    }
}

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

/// Un morceau a mesurer avant (ou pendant) sa lecture.
pub(crate) struct OfflineJob {
    pub(crate) track_id: u64,
    /// Octets complets du morceau (conteneur d'origine, non decode).
    pub(crate) data: Arc<Vec<u8>>,
    pub(crate) target_lufs: f32,
    /// Atomic de gain du morceau, quand il est deja construit (chemin
    /// gapless : la source du suivant existe avant la bascule).
    pub(crate) gain_atomic: Option<Arc<AtomicU32>>,
    /// Passe a vrai quand le morceau prend l'antenne. Le resultat n'est
    /// applique au gain que s'il arrive AVANT : jamais de correction de
    /// volume en cours de lecture.
    pub(crate) started: Option<Arc<AtomicBool>>,
}

impl OfflineJob {
    /// Le resultat doit-il etre pose sur le gain, ou seulement mis en cache ?
    ///
    /// Uniquement si le morceau n'a pas encore commence : une correction de
    /// volume en cours de lecture s'entend, et c'est precisement ce qu'on
    /// cherche a supprimer.
    pub(crate) fn should_apply(&self) -> bool {
        match (&self.gain_atomic, &self.started) {
            (Some(_), Some(started)) => !started.load(Ordering::SeqCst),
            _ => false,
        }
    }

    /// Mesure destinee au cache seul (morceau deja commence, ou qui sera
    /// demarre plus tard par un autre chemin — il lira le cache).
    pub(crate) fn cache_only(track_id: u64, data: Arc<Vec<u8>>, target_lufs: f32) -> Self {
        Self {
            track_id,
            data,
            target_lufs,
            gain_atomic: None,
            started: None,
        }
    }

    /// Mesure du prochain morceau, dont la source est deja construite : si
    /// elle aboutit a temps, le gain est pose avant la premiere note.
    pub(crate) fn for_pending(
        track_id: u64,
        data: Arc<Vec<u8>>,
        target_lufs: f32,
        gain_atomic: Arc<AtomicU32>,
        started: Arc<AtomicBool>,
    ) -> Self {
        Self {
            track_id,
            data,
            target_lufs,
            gain_atomic: Some(gain_atomic),
            started: Some(started),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(started: bool) -> OfflineJob {
        OfflineJob::for_pending(
            1,
            Arc::new(Vec::new()),
            -14.0,
            Arc::new(AtomicU32::new(0)),
            Arc::new(AtomicBool::new(started)),
        )
    }

    #[test]
    fn pose_le_gain_avant_le_debut_du_morceau() {
        assert!(job(false).should_apply());
    }

    #[test]
    fn jamais_de_correction_une_fois_le_morceau_commence() {
        assert!(!job(true).should_apply());
    }

    #[test]
    fn un_travail_de_cache_ne_touche_jamais_au_gain() {
        let j = OfflineJob::cache_only(1, Arc::new(Vec::new()), -14.0);
        assert!(!j.should_apply());
    }
}

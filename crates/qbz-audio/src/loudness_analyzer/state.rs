use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use crate::loudness_meter::LoudnessMeter;

/// Delai avant le gain provisoire. Assez pour que la loudness court-terme
/// ait de la matiere, assez tot pour que la correction tombe dans l'intro
/// plutot qu'en plein milieu du morceau (l'ancien seuil etait 10 s).
const PROVISIONAL_SECS: u64 = 2;
/// Premiere mesure integree — pour le cache uniquement.
const INTEGRATED_SECS: u64 = 10;
/// Puis raffinement, toujours pour le cache uniquement.
const REFINEMENT_SECS: u64 = 5;

pub(super) struct AnalyzerState {
    pub(super) track_id: u64,
    pub(super) target_lufs: f32,
    pub(super) meter: LoudnessMeter,
    pub(super) channels: u16,
    pub(super) sample_rate: u32,
    /// Atomic partage — ecrit par nous, lu par `DynamicAmplify`.
    pub(super) gain_atomic: Option<Arc<AtomicU32>>,
    /// Vrai des qu'un gain a ete pose pour ce morceau (cache, pre-analyse ou
    /// provisoire). Une fois vrai, plus aucun changement de volume en cours
    /// de lecture : les mesures suivantes ne nourrissent que le cache.
    pub(super) gain_applied: bool,
    pub(super) frames_at_last_measure: u64,
    pub(super) provisional_frames: u64,
    pub(super) integrated_frames: u64,
    pub(super) refinement_frames: u64,
}

impl AnalyzerState {
    pub(super) fn new(
        track_id: u64,
        sample_rate: u32,
        channels: u16,
        target_lufs: f32,
    ) -> Option<Self> {
        let meter = LoudnessMeter::new(sample_rate, channels)?;
        let fps = sample_rate as u64;
        Some(Self {
            track_id,
            target_lufs,
            meter,
            channels,
            sample_rate,
            gain_atomic: None,
            gain_applied: false,
            frames_at_last_measure: 0,
            provisional_frames: fps * PROVISIONAL_SECS,
            integrated_frames: fps * INTEGRATED_SECS,
            refinement_frames: fps * REFINEMENT_SECS,
        })
    }

    /// Repart de zero apres un seek.
    ///
    /// `gain_applied` n'est PAS remis a false : le morceau garde le volume
    /// qu'il a. Remesurer apres un seek produisait un second saut, calcule
    /// sur un extrait arbitraire du morceau.
    pub(super) fn reset_analyzer(&mut self) {
        self.meter.reset(self.sample_rate, self.channels);
        self.frames_at_last_measure = 0;
    }
}

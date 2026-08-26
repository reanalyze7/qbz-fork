//! Mesure de loudness EBU R128 reutilisable, sans thread ni I/O.
//!
//! Existe pour que la mesure temps reel (analyseur du thread audio) et la
//! mesure hors-ligne (morceau entier, avant lecture) partagent exactement le
//! meme calcul. Le decodage reste chez l'appelant : `qbz-player` a le seul
//! decodeur qui gere le CMAF/mp4 de Qobuz, et `qbz-audio` ne doit pas en
//! dependre.

use ebur128::{EbuR128, Mode};

use crate::loudness::gain::is_plausible_lufs;

pub struct LoudnessMeter {
    ebur128: EbuR128,
    channels: u16,
    peak: f32,
    frames_fed: u64,
}

impl LoudnessMeter {
    /// Mesure integree (I) et court-terme (S) : la seconde sert au gain
    /// provisoire des premieres secondes, quand l'integree n'a pas encore
    /// assez de matiere.
    pub fn new(sample_rate: u32, channels: u16) -> Option<Self> {
        let ebur128 = EbuR128::new(channels as u32, sample_rate, Mode::I | Mode::S).ok()?;
        Some(Self {
            ebur128,
            channels,
            peak: 0.0,
            frames_fed: 0,
        })
    }

    /// Ajoute des echantillons entrelaces. Renvoie false si le moteur R128
    /// les refuse (format incoherent).
    pub fn feed(&mut self, samples: &[f32]) -> bool {
        if samples.is_empty() {
            return true;
        }
        if self.ebur128.add_frames_f32(samples).is_err() {
            return false;
        }
        for s in samples {
            let a = s.abs();
            if a > self.peak {
                self.peak = a;
            }
        }
        self.frames_fed += (samples.len() / self.channels.max(1) as usize) as u64;
        true
    }

    /// Nombre de trames (echantillons par canal) ingerees.
    pub fn frames_fed(&self) -> u64 {
        self.frames_fed
    }

    /// Pic absolu observe.
    pub fn peak(&self) -> f32 {
        self.peak
    }

    /// Loudness integree, `None` si elle n'est pas exploitable.
    pub fn integrated_lufs(&self) -> Option<f32> {
        self.ebur128
            .loudness_global()
            .ok()
            .map(|l| l as f32)
            .filter(|l| is_plausible_lufs(*l))
    }

    /// Loudness court-terme (fenetre glissante de 3 s), meme filtre.
    pub fn shortterm_lufs(&self) -> Option<f32> {
        self.ebur128
            .loudness_shortterm()
            .ok()
            .map(|l| l as f32)
            .filter(|l| is_plausible_lufs(*l))
    }

    /// Repart de zero (apres un seek) en gardant le format.
    pub fn reset(&mut self, sample_rate: u32, channels: u16) {
        if let Ok(e) = EbuR128::new(channels as u32, sample_rate, Mode::I | Mode::S) {
            self.ebur128 = e;
        }
        self.channels = channels;
        self.frames_fed = 0;
        self.peak = 0.0;
    }
}

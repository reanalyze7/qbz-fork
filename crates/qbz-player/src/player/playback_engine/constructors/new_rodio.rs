//! `PlaybackEngine::new_rodio`.

use super::super::PlaybackEngine;
use rodio::{mixer::Mixer, Player as RodioPlayer};

impl PlaybackEngine {
    /// Create Rodio engine
    pub fn new_rodio(mixer: &Mixer) -> Result<Self, String> {
        let sink = RodioPlayer::connect_new(mixer);
        Ok(Self::Rodio {
            sink,
            mixer: mixer.clone(),
        })
    }
}

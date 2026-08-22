use super::*;

impl Player {
    pub(crate) fn begin_play(&self) -> u64 {
        self.state.begin_play()
    }

    /// A passing check is a snapshot, not a lock: a newer intent can still
    /// begin between this check and the subsequent AudioCommand send. That
    /// residual window is accepted; closing it would require the generation
    /// to travel inside the audio command itself.
    pub(crate) fn is_current_play(&self, gen: u64) -> bool {
        self.state.is_current_play(gen)
    }
}

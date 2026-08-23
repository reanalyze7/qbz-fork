use super::tunables::{WEIGHT_FAVORITE, WEIGHT_OPEN, WEIGHT_PLAY};

/// A user interaction with a search-surfaced entity. The weight is the score
/// increment applied to that entity for the originating query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionAction {
    /// Opened / navigated to the entity (album page, artist page, ...).
    Open,
    /// Started playback of the entity.
    Play,
    /// Favorited the entity.
    Favorite,
}

impl InteractionAction {
    /// The score increment for this action.
    pub fn weight(self) -> i64 {
        match self {
            InteractionAction::Open => WEIGHT_OPEN,
            InteractionAction::Play => WEIGHT_PLAY,
            InteractionAction::Favorite => WEIGHT_FAVORITE,
        }
    }
}

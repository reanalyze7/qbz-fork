//! Async worker-thread loaders that run a combined Qobuz search (+
//! local-library search + blacklist filtering) and map it to plain `Send`
//! data.

mod cortinilla;
mod immersive;
mod search;

pub use cortinilla::load_cortinilla;
pub use search::load_search;

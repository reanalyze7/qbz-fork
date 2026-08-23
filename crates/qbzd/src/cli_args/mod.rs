mod cmd;
mod root;
mod scrobble_sub;
mod settings_sub;
mod sub;

pub use cmd::Cmd;
pub use root::Cli;
pub use scrobble_sub::{ConfigCmd, ScrobbleCmd, ScrobbleLoginCmd};
pub use settings_sub::SettingsCmd;
pub use sub::{FavCmd, PlaylistCmd, QueueCmd, RecoCmd};

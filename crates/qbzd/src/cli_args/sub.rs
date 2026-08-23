use clap::Subcommand;

#[derive(Subcommand)]
pub enum QueueCmd {
    List  { #[arg(long)] json: bool },
    Add   { track_id: u64, #[arg(long)] next: bool },
    Remove{ index: usize },
    Clear { #[arg(long)] keep_current: bool },
    /// Reorder a 1-based position to another
    Move  { from: usize, to: usize },
    /// Jump to (play) a 1-based position
    Jump  { position: usize },
    /// Stop after the current track (or `off` to clear)
    StopAfter { arg: Option<String> },
}

#[derive(Subcommand)]
pub enum RecoCmd {
    Playlist {
        id: u64,
        #[arg(long)] limit: Option<u32>,
        #[arg(long)] ids: bool,
        #[arg(long)] json: bool,
    },
}

#[derive(Subcommand)]
pub enum FavCmd {
    List {
        #[arg(long = "type")] kind: Option<String>,
        #[arg(long)] ids: bool,
        #[arg(long)] json: bool,
    },
    Add    { fav_type: String, id: Option<String>, #[arg(long)] current: bool },
    Remove { fav_type: String, id: String },
}

#[derive(Subcommand)]
pub enum PlaylistCmd {
    List { #[arg(long)] json: bool },
    Show { id: u64, #[arg(long)] ids: bool, #[arg(long)] json: bool },
    /// Create a playlist
    Create { name: String, #[arg(long)] desc: Option<String>, #[arg(long)] public: bool },
    /// Rename / re-describe / change visibility
    Edit {
        id: u64,
        #[arg(long)] name: Option<String>,
        #[arg(long)] desc: Option<String>,
        #[arg(long)] public: bool,
        #[arg(long)] private: bool,
    },
    /// Delete an owned playlist (requires --yes)
    Rm { id: u64, #[arg(long)] yes: bool },
    /// Add tracks (ids, or - to read from stdin)
    Add { id: u64, track_ids: Vec<String> },
    /// Remove tracks (plain track ids)
    Remove { id: u64, track_ids: Vec<String> },
}

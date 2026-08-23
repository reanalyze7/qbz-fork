// ============================ Scrobbler (CONSOLE ext) ============================

pub const SCROBBLER_TITLE: &str = "Scrobbler";
pub const HELP_SCROBBLER: &str = "L connect Last.fm · B connect ListenBrainz · Tab nav · Esc nav · q quit";
// Alt-screen handoffs — printed on the plain terminal before the CLI auth flow
// runs (same methodology as the Account browser login).
pub const SCROBBLE_LASTFM_HANDOFF: &str =
    "Connecting Last.fm — a browser authorize step follows below.\n";
pub const SCROBBLE_LISTENBRAINZ_HANDOFF: &str =
    "Connecting ListenBrainz — paste your user token below.\n";
pub const SCROBBLE_RETURN_HINT: &str = "\nPress Enter to return to setup…";

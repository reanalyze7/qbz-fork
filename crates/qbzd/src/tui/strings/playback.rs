// ============================ Playback (§3.3) ============================

pub const PLAYBACK_TITLE: &str = "Playback";
pub const PLAYBACK_GROUP_QUALITY: &str = "QUALITY";
pub const PLAYBACK_GROUP_BEHAVIOR: &str = "BEHAVIOR";
pub const PLAYBACK_GROUP_SESSION: &str = "SESSION";
pub const PLAYBACK_GROUP_CONTROLS: &str = "MEDIA CONTROLS";

pub const P_QUALITY: &str = "Streaming quality";
pub const P_LIMIT_DEVICE: &str = "Limit quality to device";
pub const P_MAX_RATE: &str = "Maximum sample rate";
pub const P_ALLOW_FALLBACK: &str = "Allow quality fallback";
pub const P_RETRY_FAIL: &str = "When retries fail";
pub const P_CONTINUE: &str = "Continue after track";
pub const P_GAPLESS: &str = "Gapless playback";
pub const P_RESTORE: &str = "Restore session";
pub const P_RESUME_POS: &str = "Resume position";
pub const P_MPRIS: &str = "System media controls";
pub const P_MPRIS_DESC: &str = "MPRIS: KDE/GNOME media widget + media keys · restart to apply";

pub const R_LIMIT_OFF: &str = "Limit quality to device off";
pub const R_STREAMING_ONLY_ON: &str = "off while Audio > Streaming only on";
pub const R_RESTORE_OFF: &str = "needs Restore session";

pub const Q_MP3: &str = "MP3";
pub const Q_CD: &str = "CD Quality";
pub const Q_HIRES: &str = "Hi-Res";
pub const Q_HIRES_PLUS: &str = "Hi-Res+";

pub const RETRY_FALLBACK: &str = "Fall back (play lowest available)";
pub const RETRY_SKIP: &str = "Skip the track";
/// Stored `ask` render until the operator picks (§3.3.2). The TUI never writes `ask`.
pub const RETRY_ASK: &str = "Ask (desktop setting) — daemon falls back";

pub const AUTOPLAY_ON: &str = "on";
pub const AUTOPLAY_OFF: &str = "off";
/// A pre-existing `infinite` (radio, P1) renders read-only until toggled (§3.3.1).
pub const AUTOPLAY_INFINITE: &str = "on (infinite radio)";

pub const RATE_NO_LIMIT: &str = "No limit";

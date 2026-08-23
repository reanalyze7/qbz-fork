// ============================ shell / navigation ============================

/// Header title (accent-bold, left of the version). One row, always visible.
pub const APP_TITLE: &str = "QBZ Daemon Setup";
pub const HELP_TITLE: &str = "Help";

/// Breadcrumb root node (dim `Setup ›` prefix). The current node (section or,
/// while editing, the field label) carries the accent.
pub const BREADCRUMB_ROOT: &str = "Setup";

/// Persistent left-nav labels (fixed order), COMPACT tier (< 100 cols). NAME
/// only; Dirty-capable sections (Audio/Playback/Network) stay ≤ 8 chars
/// so a trailing `*` fits the 14-col sidebar; Account/Import/Wizard are never
/// dirty. Seven since FB4 added the HiFi Wizard (owner-sanctioned cap break over
/// the old six-screen D7 cap).
pub const SIDEBAR_LABELS: [&str; 7] = [
    "Account",
    "Audio",
    "Playback",
    "Network",
    "Import/Exp",
    "Wizard",
    "Scrobbler",
];

/// Left-nav labels, WIDE tier (≥ 100 cols → the 28-col sidebar, FB5). The extra
/// room lets `Import / Export` spell itself out; everything else is already whole.
pub const SIDEBAR_LABELS_WIDE: [&str; 7] = [
    "Account",
    "Audio",
    "Playback",
    "Network",
    "Import / Export",
    "Wizard",
    "Scrobbler",
];

/// A terse, static one-line summary shown dim UNDER each name in the wide sidebar
/// only (FB5). Static (not live device state) — it names what the section is for
/// so the roomy sidebar reads intentionally; it never needs the section loaded.
pub const SIDEBAR_SUMMARIES: [&str; 8] = [
    "sign in",
    "output · bit-perfect",
    "quality · behavior",
    "cast target",
    "http control",
    "settings bundle",
    "DAC setup",
    "last.fm · listenbrainz",
];

// Global help-bar hints (context-sensitive; assembled per focus + screen).
pub const HELP_NAV: &str = "up/down move · Enter open · 1-8 jump · Tab content · ? help · q quit";
pub const HELP_CONTENT_CLEAN: &str = "up/down move · Enter edit · Tab nav · Esc nav · ? help · q quit";
pub const HELP_CONTENT_DIRTY: &str = "up/down move · Enter edit · s SAVE* · Tab nav · Esc nav · q quit";
pub const HELP_AUDIO_CLEAN: &str =
    "up/down move · Enter edit · r refresh · / filter · Tab nav · Esc nav";
pub const HELP_AUDIO_DIRTY: &str =
    "up/down move · s SAVE* · r refresh · / filter · Tab nav · Esc nav";
pub const HELP_SELECT: &str = "up/down choose · Enter select · Esc cancel";
pub const HELP_FILTER: &str = "type to filter · up/down choose · Enter select · Esc cancel";
pub const HELP_INPUT: &str = "type · Enter accept · Esc cancel";

pub const HELP_OVERLAY: &str = "GLOBAL KEYS

  up / down (or j / k)   move (sidebar or field)
  Enter                  open a section / edit a field / confirm
  Tab                    toggle sidebar <-> content
  Esc                    content: back to sidebar · sidebar: quit
  1 - 7                  jump straight to a section
  s                      save the current section
  r                      refresh (Audio: re-enumerate devices)
  /                      filter (device picker)
  ?                      this help
  q                      quit (asks to save unsaved changes)

Each section saves explicitly with 's'. A '*' by a section name means unsaved
edits. Leaving a dirty section asks to save first. The daemon does NOT need to
be running — changes apply when it next starts.

  Press Esc or ? to close.";

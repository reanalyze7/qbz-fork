//! Action model: `Category`, `Context`, `ActionDef`, the `ACTIONS` table.

/// Display/grouping category. Order here is the on-screen order.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Playback,
    Navigation,
    Ui,
}

impl Category {
    pub(super) const ORDER: [Category; 3] = [
        Category::Playback,
        Category::Navigation,
        Category::Ui,
    ];

    /// English source string for the localized category header.
    pub(super) fn label_en(self) -> &'static str {
        match self {
            Category::Playback => "Playback",
            Category::Navigation => "Navigation",
            Category::Ui => "Interface",
        }
    }
}

/// When an action only fires in a specific surface.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Context {
    None,
}

pub struct ActionDef {
    pub id: &'static str,
    pub label_en: &'static str,
    pub category: Category,
    pub default: &'static str,
    pub context: Context,
}

/// The full action table — a 1:1 port of the Tauri `ACTIONS` array.
pub const ACTIONS: &[ActionDef] = &[
    // Playback
    ActionDef { id: "playback.toggle", label_en: "Play / Pause", category: Category::Playback, default: "Space", context: Context::None },
    ActionDef { id: "playback.next", label_en: "Next Track", category: Category::Playback, default: "Ctrl+ArrowRight", context: Context::None },
    ActionDef { id: "playback.prev", label_en: "Previous Track", category: Category::Playback, default: "Ctrl+ArrowLeft", context: Context::None },
    // Navigation
    ActionDef { id: "nav.back", label_en: "Go Back", category: Category::Navigation, default: "Alt+ArrowLeft", context: Context::None },
    ActionDef { id: "nav.forward", label_en: "Go Forward", category: Category::Navigation, default: "Alt+ArrowRight", context: Context::None },
    ActionDef { id: "nav.search", label_en: "Search", category: Category::Navigation, default: "Ctrl+f", context: Context::None },
    ActionDef { id: "nav.settings", label_en: "Settings", category: Category::Navigation, default: "Ctrl+,", context: Context::None },
    // Interface
    ActionDef { id: "ui.sidebar", label_en: "Toggle Sidebar", category: Category::Ui, default: "Shift+S", context: Context::None },
    ActionDef { id: "ui.queue", label_en: "Queue", category: Category::Ui, default: "q", context: Context::None },
    ActionDef { id: "ui.escape", label_en: "Close / Dismiss", category: Category::Ui, default: "Escape", context: Context::None },
    ActionDef { id: "ui.showShortcuts", label_en: "Show Shortcuts", category: Category::Ui, default: "?", context: Context::None },
    ActionDef { id: "ui.openLink", label_en: "Open Qobuz Link", category: Category::Ui, default: "Ctrl+l", context: Context::None },
    // Seek
    ActionDef { id: "focus.seekForward", label_en: "Seek Forward (5s)", category: Category::Playback, default: "ArrowRight", context: Context::None },
    ActionDef { id: "focus.seekBack", label_en: "Seek Back (5s)", category: Category::Playback, default: "ArrowLeft", context: Context::None },
    ActionDef { id: "focus.seekForwardLong", label_en: "Seek Forward (10s)", category: Category::Playback, default: "Shift+ArrowRight", context: Context::None },
    ActionDef { id: "focus.seekBackLong", label_en: "Seek Back (10s)", category: Category::Playback, default: "Shift+ArrowLeft", context: Context::None },
];

pub(super) fn action(id: &str) -> Option<&'static ActionDef> {
    ACTIONS.iter().find(|a| a.id == id)
}

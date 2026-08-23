#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoveryTab {
    Home,
    EditorPicks,
    ForYou,
}

impl DiscoveryTab {
    /// JSON / persistence key (matches the Tauri localStorage object keys).
    pub fn as_key(&self) -> &'static str {
        match self {
            DiscoveryTab::Home => "home",
            DiscoveryTab::EditorPicks => "editorPicks",
            DiscoveryTab::ForYou => "forYou",
        }
    }

    pub fn from_key(s: &str) -> Option<Self> {
        match s {
            "home" => Some(DiscoveryTab::Home),
            "editorPicks" => Some(DiscoveryTab::EditorPicks),
            "forYou" => Some(DiscoveryTab::ForYou),
            _ => None,
        }
    }

    pub const ALL: [DiscoveryTab; 3] =
        [DiscoveryTab::Home, DiscoveryTab::EditorPicks, DiscoveryTab::ForYou];
}

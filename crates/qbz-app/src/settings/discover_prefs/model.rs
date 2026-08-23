use super::section_id::DiscoverySectionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionPref {
    pub id: DiscoverySectionId,
    pub enabled: bool,
}

pub(super) const fn pref(id: DiscoverySectionId, enabled: bool) -> SectionPref {
    SectionPref { id, enabled }
}

/// The per-tab ordered preference lists. Field order is irrelevant; the Vec
/// order within each tab is the render order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverPrefs {
    pub home: Vec<SectionPref>,
    pub editor_picks: Vec<SectionPref>,
    pub for_you: Vec<SectionPref>,
    /// Opt-out: show the external "Recommendations" tab in Discover. Default on.
    pub show_recommendations: bool,
    /// Recommendations results-cache window, in hours. One of {24,36,48,72};
    /// drives how long the built reco rows are served from cache before a
    /// rebuild. Default 48.
    pub reco_cache_ttl_hours: i64,
}

pub use super::defaults::default_prefs;

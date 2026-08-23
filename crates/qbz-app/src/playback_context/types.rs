use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    Album,
    Playlist,
    ArtistTop,
    LabelTop,
    HomeList,
    DailyQ,
    WeeklyQ,
    FavQ,
    TopQ,
    Favorites,
    LocalLibrary,
    Radio,
    Search,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentSource {
    Qobuz,
    Local,
}

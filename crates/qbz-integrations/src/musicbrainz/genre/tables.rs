//! Static lookup tables used for genre/tag normalization and filtering.

/// Tags that provide no useful genre/scene signal
pub(super) const NOISY_TAGS: &[&str] = &[
    "favorites",
    "favorite",
    "favourite",
    "favourites",
    "awesome",
    "seen live",
    "cool",
    "good",
    "great",
    "love",
    "loved",
    "amazing",
    "best",
    "american",
    "british",
    "canadian",
    "australian",
    "german",
    "french",
    "japanese",
    "swedish",
    "norwegian",
    "finnish",
    "irish",
    "scottish",
    "korean",
    "mexican",
    "brazilian",
    "colombian",
    "argentinian",
    "chilean",
    "spanish",
    "italian",
    "dutch",
    "russian",
    "chinese",
    "indian",
    "african",
    "female vocalists",
    "male vocalists",
    "female vocalist",
    "male vocalist",
    "singer-songwriter",
    "bands i've seen live",
    "bands i have seen live",
    "check out",
    "spotify",
    "under 2000 listeners",
];

/// Overly broad genre tags that return too much noise when used as search queries.
/// These are excluded from the search genres list but kept for affinity scoring.
/// Example: In Mexico everything is "latin", in UK everything is "rock" — searching
/// by these returns the entire country's catalog instead of scene-relevant artists.
pub(super) const BROAD_TAGS: &[&str] = &[
    "rock",
    "pop",
    "latin",
    "latin music",
    "electronic",
    "electronica",
    "jazz",
    "metal",
    "hip hop",
    "hip-hop",
    "hiphop",
    "rap",
    "classical",
    "classical music",
    "folk",
    "folk music",
    "blues",
    "blues music",
    "country",
    "country music",
    "soul",
    "funk",
    "r&b",
    "rnb",
    "reggae",
    "world",
    "world music",
    "experimental",
    "dance",
    "dance music",
    "soundtrack",
    "instrumental",
];

/// Minimum vote count to consider a tag as a primary genre
pub(super) const GENRE_MIN_VOTES: i32 = 1;

/// Maximum number of primary genres to extract
pub(super) const MAX_GENRES: usize = 5;

/// Maximum number of secondary tags to keep
pub(super) const MAX_TAGS: usize = 10;

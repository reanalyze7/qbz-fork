//! Incompatible-genre keyword tables used by `has_incompatible_genre`.
//!
//! These are logically a single data table split out only because the
//! combined function + data exceeded the file-length budget; keep both
//! arrays together in this one file.

// Incompatible genre keywords - these would never appear in a rock/metal context
// NOTE: We force English locale when fetching, so only English names needed
pub(super) const INCOMPATIBLE_GENRES: &[&str] = &[
    // Latin/Tropical
    "bachata",
    "merengue",
    "reggaeton",
    "salsa",
    "cumbia",
    "vallenato",
    "latin pop",
    "latin music",
    "tropical",
    "urbano",
    "regional mexican",
    "latin", // Generic Latin parent genre
    // Asian pop
    "k-pop",
    "kpop",
    "j-pop",
    "jpop",
    "mandopop",
    "cantopop",
    "c-pop",
    // European folk/schlager
    "schlager",
    "chanson",
    "french chanson",
    "volksmusik",
    // Religious
    "gospel",
    "christian",
    "worship",
    "religious",
    "spiritual",
    // Children/Family
    "children",
    "nursery",
    "lullaby",
    "kids",
    // Electronic/Dance (club-oriented)
    "trance",
    "techno",
    "house",
    "edm",
    "dubstep",
    "drum and bass",
    "hardstyle",
    "eurodance",
    "hands up",
    "happy hardcore",
    "dance",
    // Spoken word/Non-music
    "audiobook",
    "spoken word",
    "podcast",
    "meditation",
    "asmr",
    "relaxation",
    "sleep",
    "nature sounds",
    "white noise",
    "comedy",
    "stand-up",
    // Country (usually incompatible with metal)
    "country",
    "bluegrass",
    "americana",
    // New age/Wellness
    "new age",
    "healing",
    "spa",
    "yoga",
    "mindfulness",
    "wellness",
];

// Additional title-based checks for non-music content
pub(super) const INCOMPATIBLE_TITLE_KEYWORDS: &[&str] = &[
    "audiobook",
    "hörbuch",
    "hörspiel",
    "gelesen von",
    "read by",
    "narrated by",
    "lesung",
    "märchen",
    "fairy tale",
    "meditation",
    "relaxation",
    "sleep music",
    "yoga music",
    "trance mix",
    "club mix",
    "dance mix",
    "dj mix",
];

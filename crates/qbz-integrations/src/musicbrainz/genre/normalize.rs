//! Genre/tag name normalization and noise/broadness filtering.

use super::tables::{BROAD_TAGS, NOISY_TAGS};

/// Normalize genre/tag name to canonical form
pub fn normalize_genre(name: &str) -> String {
    let lower = name.to_lowercase().trim().to_string();

    match lower.as_str() {
        // Rock variants
        "alt rock" | "alt-rock" | "alternative" => "alternative rock".to_string(),
        "grunge rock" => "grunge".to_string(),
        "prog" | "prog rock" | "progressive" => "progressive rock".to_string(),
        "prog metal" | "progressive metal" => "progressive metal".to_string(),
        "post punk" => "post-punk".to_string(),
        "post rock" => "post-rock".to_string(),
        "post metal" => "post-metal".to_string(),
        "math rock" => "math rock".to_string(),
        "stoner" | "stoner rock" => "stoner rock".to_string(),
        "psychedelic" | "psych" | "psych rock" => "psychedelic rock".to_string(),
        "shoegaze" | "shoe gaze" => "shoegaze".to_string(),
        "noise rock" | "noise-rock" => "noise rock".to_string(),
        "hard rock" | "hard-rock" => "hard rock".to_string(),
        "indie" => "indie rock".to_string(),
        "punk" => "punk rock".to_string(),

        // Metal variants
        "death metal" | "death-metal" => "death metal".to_string(),
        "black metal" | "black-metal" => "black metal".to_string(),
        "doom" | "doom metal" | "doom-metal" => "doom metal".to_string(),
        "thrash" | "thrash metal" => "thrash metal".to_string(),
        "metalcore" | "metal core" => "metalcore".to_string(),
        "nu metal" | "nu-metal" | "nü-metal" => "nu metal".to_string(),
        "sludge" | "sludge metal" => "sludge metal".to_string(),

        // Electronic variants
        "electronic" | "electronica" => "electronic".to_string(),
        "idm" | "intelligent dance music" => "idm".to_string(),
        "edm" => "electronic dance music".to_string(),
        "dnb" | "drum n bass" | "drum & bass" | "drum'n'bass" => "drum and bass".to_string(),
        "ambient" | "ambient music" => "ambient".to_string(),
        "synth pop" | "synth-pop" | "synthpop" => "synthpop".to_string(),
        "trip hop" | "trip-hop" => "trip-hop".to_string(),
        "downtempo" | "down tempo" => "downtempo".to_string(),
        "techno" | "detroit techno" => "techno".to_string(),
        "house" | "house music" => "house".to_string(),

        // Hip-hop variants
        "hip hop" | "hip-hop" | "hiphop" => "hip hop".to_string(),
        "rap" | "rap music" => "hip hop".to_string(),
        "trap" | "trap music" => "trap".to_string(),

        // Jazz variants
        "jazz" | "contemporary jazz" => "jazz".to_string(),
        "jazz fusion" | "fusion" => "jazz fusion".to_string(),
        "free jazz" | "free-jazz" => "free jazz".to_string(),
        "acid jazz" | "acid-jazz" => "acid jazz".to_string(),

        // R&B / Soul
        "r&b" | "rnb" | "rhythm and blues" => "r&b".to_string(),
        "neo soul" | "neo-soul" => "neo-soul".to_string(),

        // Other
        "folk" | "folk music" => "folk".to_string(),
        "country" | "country music" => "country".to_string(),
        "blues" | "blues music" => "blues".to_string(),
        "classical" | "classical music" => "classical".to_string(),
        "world" | "world music" => "world music".to_string(),
        "reggae" | "reggae music" => "reggae".to_string(),
        "ska" | "ska music" => "ska".to_string(),
        "latin" | "latin music" => "latin".to_string(),

        // Default: return as-is (lowercased)
        _ => lower,
    }
}

/// Check if a tag is noisy (provides no genre/scene signal)
pub(super) fn is_noisy_tag(tag: &str) -> bool {
    let lower = tag.to_lowercase();
    NOISY_TAGS.iter().any(|noisy| lower == *noisy)
}

/// Check if a normalized genre is too broad to use as a search query.
/// These tags are kept for affinity scoring but excluded from MB search queries.
pub fn is_broad_genre(normalized: &str) -> bool {
    BROAD_TAGS.iter().any(|broad| normalized == *broad)
}

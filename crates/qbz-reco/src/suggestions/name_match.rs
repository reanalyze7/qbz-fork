//! Pure string helpers for fuzzy artist-name matching.

use std::collections::HashSet;

/// Normalize a name for comparison (remove accents, lowercase)
pub(super) fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace('ú', "u")
        .replace('à', "a")
        .replace('è', "e")
        .replace('ì', "i")
        .replace('ò', "o")
        .replace('ù', "u")
        .replace('ä', "a")
        .replace('ë', "e")
        .replace('ï', "i")
        .replace('ö', "o")
        .replace('ü', "u")
        .replace('ñ', "n")
        .replace('ç', "c")
}

/// Check if two artist names are similar enough to be considered a match
///
/// STRICT matching to prevent false positives like:
/// - "Martín Méndez" matching "Tomas Martin Lopez" (share "Martin")
/// - "Martín Méndez" matching "Martin Mendez" (different person, same name)
///
/// For person names (2-3 words), we require ALL words to match.
/// This handles "George Harrison" vs "Harrison, George" but rejects partial matches.
pub(super) fn names_similar(name1: &str, name2: &str) -> bool {
    let norm1 = normalize_name(name1);
    let norm2 = normalize_name(name2);

    // Exact match after normalization
    if norm1 == norm2 {
        return true;
    }

    // Split into words
    let words1: HashSet<&str> = norm1.split_whitespace().collect();
    let words2: HashSet<&str> = norm2.split_whitespace().collect();

    if words1.is_empty() || words2.is_empty() {
        return false;
    }

    // Count matching words
    let matches = words1.intersection(&words2).count();
    let max_words = words1.len().max(words2.len());
    let min_words = words1.len().min(words2.len());

    // VERY STRICT for person names:
    // - For 2-word names: require EXACT same words (handles "George Harrison" vs "Harrison, George")
    // - For 3-word names: allow at most 1 extra word
    // - This rejects "Martin Lopez" vs "Tomas Martin Lopez" (different people)
    if min_words == 2 {
        // For 2-word names, require EXACTLY the same words (just different order allowed)
        // "Martin Lopez" vs "Tomas Martin Lopez" -> max_words=3, min_words=2 -> REJECT
        // "George Harrison" vs "Harrison, George" -> max_words=2, min_words=2 -> ACCEPT
        matches == min_words && max_words == min_words
    } else if min_words == 3 {
        // For 3-word names, allow at most 1 extra word
        matches >= min_words && (max_words - min_words) <= 1
    } else {
        // For longer names (bands, etc.), allow some flexibility
        matches as f32 / max_words as f32 >= 0.75
    }
}

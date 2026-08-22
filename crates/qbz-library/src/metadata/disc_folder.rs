//! Disc-folder name detection: is a folder name like "Disc 1" / "CD2" a
//! standalone disc subfolder, as opposed to an album title that merely
//! contains a disc-like word (issue #147)?

use super::MetadataExtractor;

impl MetadataExtractor {
    pub(super) fn is_disc_folder(name: &str) -> bool {
        let lower = name.to_lowercase();
        let tokens: Vec<&str> = lower
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|t| !t.is_empty())
            .collect();

        // Must contain at least one disc-related token
        let has_disc_token = tokens.iter().any(|token| {
            *token == "disc"
                || *token == "disk"
                || *token == "cd"
                || (token.starts_with("disc")
                    && token[4..].chars().all(|c| c.is_ascii_digit())
                    && !token[4..].is_empty())
                || (token.starts_with("disk")
                    && token[4..].chars().all(|c| c.is_ascii_digit())
                    && !token[4..].is_empty())
                || (token.starts_with("cd")
                    && token[2..].chars().all(|c| c.is_ascii_digit())
                    && !token[2..].is_empty())
        });

        if !has_disc_token {
            return false;
        }

        // A genuine disc folder name consists ONLY of disc-related tokens,
        // digits, and common modifiers like "bonus". If other words remain
        // after filtering these out, the name is an album title that happens
        // to contain "Disc 1" etc., not a standalone disc folder.
        // Examples that ARE disc folders: "Disc 1", "CD2", "Bonus Disc", "disc01"
        // Examples that are NOT: "Relaxation Disc1", "Now 75 - CD1",
        //   "100 Popular Classics, Disc 1"
        let has_extra_words = tokens.iter().any(|token| {
            // Pure digits are fine
            if token.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            // Disc keywords are fine
            if *token == "disc" || *token == "disk" || *token == "cd" {
                return false;
            }
            // Disc+number compounds are fine (disc1, cd02, etc.)
            for prefix in &["disc", "disk", "cd"] {
                if token.starts_with(prefix) {
                    let rest = &token[prefix.len()..];
                    if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                        return false;
                    }
                }
            }
            // Common disc folder modifiers are fine
            if matches!(*token, "bonus" | "extra" | "side" | "part") {
                return false;
            }
            // Anything else means this is not a pure disc folder
            true
        });

        !has_extra_words
    }

    pub(super) fn disc_number_from_name(name: &str) -> Option<u32> {
        let lower = name.to_lowercase();
        let tokens: Vec<&str> = lower
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|t| !t.is_empty())
            .collect();

        for (i, token) in tokens.iter().enumerate() {
            if (*token == "disc" || *token == "disk" || *token == "cd")
                && tokens
                    .get(i + 1)
                    .map_or(false, |t| t.chars().all(|c| c.is_ascii_digit()))
            {
                if let Some(next) = tokens.get(i + 1) {
                    if let Ok(value) = next.parse::<u32>() {
                        if value > 0 {
                            return Some(value);
                        }
                    }
                }
            }

            for prefix in ["disc", "disk", "cd"] {
                if token.starts_with(prefix) {
                    let rest = &token[prefix.len()..];
                    if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                        if let Ok(value) = rest.parse::<u32>() {
                            if value > 0 {
                                return Some(value);
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

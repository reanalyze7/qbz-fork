//! Pure title/name string parsing: stripping a trailing "(2024)" year
//! suffix or a "Disc 1" / "CD2" suffix off an album or track title.

use super::MetadataExtractor;

impl MetadataExtractor {
    pub(super) fn strip_year_suffix(name: &str) -> String {
        let trimmed = name.trim();
        for (open, close) in [("(", ")"), ("[", "]")] {
            if trimmed.ends_with(close) {
                if let Some(start) = trimmed.rfind(open) {
                    let inside = &trimmed[start + 1..trimmed.len() - 1];
                    if inside.len() == 4 && inside.chars().all(|c| c.is_ascii_digit()) {
                        return trimmed[..start].trim().to_string();
                    }
                }
            }
        }
        trimmed.to_string()
    }

    pub(super) fn strip_disc_suffix(title: &str) -> String {
        let trimmed = title.trim();

        for (open, close) in [("(", ")"), ("[", "]")] {
            if trimmed.ends_with(close) {
                if let Some(start) = trimmed.rfind(open) {
                    let inside = trimmed[start + 1..trimmed.len() - 1].trim();
                    if Self::is_disc_designator(inside) {
                        return trimmed[..start].trim().to_string();
                    }
                }
            }
        }

        let tokens: Vec<&str> = trimmed
            .split_whitespace()
            .filter(|token| *token != "-" && *token != "–" && *token != "—")
            .collect();

        if tokens.len() >= 2 {
            let last = tokens[tokens.len() - 1];
            let prev = tokens[tokens.len() - 2];
            if Self::is_disc_marker(prev) && last.chars().all(|c| c.is_ascii_digit()) {
                return tokens[..tokens.len() - 2].join(" ").trim().to_string();
            }
        }

        if let Some(last) = tokens.last() {
            if Self::is_disc_designator(last) && tokens.len() > 1 {
                return tokens[..tokens.len() - 1].join(" ").trim().to_string();
            }
        }

        trimmed.to_string()
    }

    pub(super) fn is_disc_marker(value: &str) -> bool {
        matches!(value.to_lowercase().as_str(), "disc" | "disk" | "cd")
    }

    pub(super) fn is_disc_designator(value: &str) -> bool {
        let cleaned: String = value
            .to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();

        if cleaned.starts_with("disc") {
            let rest = &cleaned[4..];
            return !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit());
        }
        if cleaned.starts_with("disk") {
            let rest = &cleaned[4..];
            return !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit());
        }
        if cleaned.starts_with("cd") {
            let rest = &cleaned[2..];
            return !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit());
        }

        false
    }
}

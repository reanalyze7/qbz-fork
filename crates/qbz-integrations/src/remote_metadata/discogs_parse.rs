//! Small pure parsers used by the Discogs converters (`discogs_convert.rs`):
//! track position and duration string parsing.

/// Parse Discogs track position to (disc_number, track_number)
/// Handles formats: "1", "A1", "1-1", "CD1-1", "1.1"
pub fn parse_discogs_position(position: &str) -> (u8, u8) {
    let position = position.trim();

    // Handle empty position
    if position.is_empty() {
        return (1, 1);
    }

    // Try "X-Y" format (e.g., "1-5", "CD1-3")
    if let Some(pos) = position.find('-') {
        let disc_part = &position[..pos];
        let track_part = &position[pos + 1..];

        // Extract number from disc part (handle "CD1", "1", etc.)
        let disc = disc_part
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u8>()
            .unwrap_or(1);

        let track = track_part.parse::<u8>().unwrap_or(1);
        return (disc, track);
    }

    // Try "X.Y" format
    if let Some(pos) = position.find('.') {
        let disc_part = &position[..pos];
        let track_part = &position[pos + 1..];

        let disc = disc_part.parse::<u8>().unwrap_or(1);
        let track = track_part.parse::<u8>().unwrap_or(1);
        return (disc, track);
    }

    // Handle vinyl sides (A, B, C, D -> disc 1, 1, 2, 2)
    if position.starts_with(|c: char| c.is_ascii_alphabetic()) {
        let side = position.chars().next().unwrap().to_ascii_uppercase();
        let track_str: String = position.chars().skip(1).collect();
        let track = track_str.parse::<u8>().unwrap_or(1);

        let disc = match side {
            'A' | 'B' => 1,
            'C' | 'D' => 2,
            'E' | 'F' => 3,
            _ => 1,
        };

        return (disc, track);
    }

    // Simple number
    let track = position.parse::<u8>().unwrap_or(1);
    (1, track)
}

/// Parse Discogs duration string to milliseconds
/// Handles format: "M:SS" or "MM:SS" or "H:MM:SS"
pub fn parse_discogs_duration(duration: &str) -> Option<u32> {
    let parts: Vec<&str> = duration.split(':').collect();

    match parts.len() {
        2 => {
            // M:SS or MM:SS
            let minutes: u32 = parts[0].parse().ok()?;
            let seconds: u32 = parts[1].parse().ok()?;
            Some((minutes * 60 + seconds) * 1000)
        }
        3 => {
            // H:MM:SS
            let hours: u32 = parts[0].parse().ok()?;
            let minutes: u32 = parts[1].parse().ok()?;
            let seconds: u32 = parts[2].parse().ok()?;
            Some((hours * 3600 + minutes * 60 + seconds) * 1000)
        }
        _ => None,
    }
}

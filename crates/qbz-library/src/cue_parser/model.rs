/// Parsed CUE sheet
#[derive(Debug, Clone)]
pub struct CueSheet {
    /// Path to the .cue file
    pub file_path: String,
    /// Referenced audio file (resolved to absolute path)
    pub audio_file: String,
    /// Album title
    pub title: Option<String>,
    /// Album performer/artist
    pub performer: Option<String>,
    /// Tracks in the CUE sheet
    pub tracks: Vec<CueTrack>,
}

/// A track within a CUE sheet
#[derive(Debug, Clone)]
pub struct CueTrack {
    /// Track number
    pub number: u32,
    /// Track title
    pub title: String,
    /// Track performer (if different from album)
    pub performer: Option<String>,
    /// Start time in seconds
    pub start_secs: f64,
}

/// CUE time format (MM:SS:FF where FF is frames, 75 frames per second)
#[derive(Debug, Clone, Copy)]
pub struct CueTime {
    pub minutes: u32,
    pub seconds: u32,
    pub frames: u32,
}

impl CueTime {
    /// Convert to seconds (frames are 1/75 second)
    pub fn to_seconds(&self) -> f64 {
        self.minutes as f64 * 60.0 + self.seconds as f64 + self.frames as f64 / 75.0
    }

    /// Parse "MM:SS:FF" format
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(CueTime {
            minutes: parts[0].parse().ok()?,
            seconds: parts[1].parse().ok()?,
            frames: parts[2].parse().ok()?,
        })
    }
}

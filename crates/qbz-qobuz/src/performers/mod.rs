//! Performers string parsing utilities
//!
//! Qobuz provides performer credits as a formatted string like:
//! "John Coltrane, Saxophone, MainArtist - McCoy Tyner, Piano - Jimmy Garrison, Double Bass"
//!
//! This module parses that string into structured data.

use serde::{Deserialize, Serialize};

mod role_labels;
mod role_labels_extra;
mod roles;

pub use roles::{format_role_label, group_credits_ordered};

/// A performer with their name and roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Performer {
    pub name: String,
    pub roles: Vec<String>,
}

impl Performer {
    pub fn new(name: String, roles: Vec<String>) -> Self {
        Self { name, roles }
    }
}

/// Parse a Qobuz performers string into structured data
///
/// Format: "Name, Role1, Role2 - Name, Role1 - Name, Role1, Role2"
///
/// # Examples
///
/// ```
/// use qbz_qobuz::performers::parse_performers;
///
/// let performers = parse_performers("John Coltrane, Saxophone, MainArtist - McCoy Tyner, Piano");
/// assert_eq!(performers.len(), 2);
/// assert_eq!(performers[0].name, "John Coltrane");
/// assert_eq!(performers[0].roles, vec!["Saxophone", "MainArtist"]);
/// ```
pub fn parse_performers(performers_str: &str) -> Vec<Performer> {
    if performers_str.is_empty() {
        return Vec::new();
    }

    performers_str
        .split(" - ")
        .filter_map(|segment| {
            let segment = segment.trim();
            if segment.is_empty() {
                return None;
            }

            let parts: Vec<&str> = segment.split(", ").collect();
            if parts.is_empty() {
                return None;
            }

            let name = parts[0].trim().to_string();
            if name.is_empty() {
                return None;
            }

            let roles: Vec<String> = parts[1..]
                .iter()
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .collect();

            Some(Performer::new(name, roles))
        })
        .collect()
}

/// Group performers by their roles
///
/// Returns a map where keys are role names and values are lists of performer names
pub fn group_by_role(performers: &[Performer]) -> std::collections::HashMap<String, Vec<String>> {
    let mut grouped: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for performer in performers {
        for role in &performer.roles {
            grouped
                .entry(role.clone())
                .or_default()
                .push(performer.name.clone());
        }
    }

    grouped
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

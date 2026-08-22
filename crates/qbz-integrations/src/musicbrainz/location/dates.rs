//! Formatting MusicBrainz life-span dates into human-readable strings

use crate::musicbrainz::LifeSpan;

/// Format a human-readable date from MB life_span
pub fn format_life_span_date(life_span: &LifeSpan, _is_person: bool) -> Option<String> {
    let begin = life_span.begin.as_deref()?;

    let begin_formatted = format_mb_date(begin);
    let ended = life_span.ended.unwrap_or(false);

    if ended {
        if let Some(end) = life_span.end.as_deref() {
            let end_formatted = format_mb_date(end);
            Some(format!("{}–{}", begin_formatted, end_formatted))
        } else {
            Some(begin_formatted)
        }
    } else {
        Some(begin_formatted)
    }
}

/// Format a MusicBrainz date string into a short human-readable form
/// Input formats: "1990", "1990-05", "1990-05-14"
fn format_mb_date(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    match parts.len() {
        1 => parts[0].to_string(),
        2 => {
            let month = match parts[1] {
                "01" => "Jan",
                "02" => "Feb",
                "03" => "Mar",
                "04" => "Apr",
                "05" => "May",
                "06" => "Jun",
                "07" => "Jul",
                "08" => "Aug",
                "09" => "Sep",
                "10" => "Oct",
                "11" => "Nov",
                "12" => "Dec",
                _ => parts[1],
            };
            format!("{} {}", month, parts[0])
        }
        3 => {
            let month = match parts[1] {
                "01" => "Jan",
                "02" => "Feb",
                "03" => "Mar",
                "04" => "Apr",
                "05" => "May",
                "06" => "Jun",
                "07" => "Jul",
                "08" => "Aug",
                "09" => "Sep",
                "10" => "Oct",
                "11" => "Nov",
                "12" => "Dec",
                _ => parts[1],
            };
            format!("{} {}", month, parts[0])
        }
        _ => date.to_string(),
    }
}

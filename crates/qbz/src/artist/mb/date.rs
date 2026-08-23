/// Format a MusicBrainz partial date into a short human string —
/// "1990", "May 1990", or "May 14, 1990" — matching Tauri's
/// formatMbDate_v2 output when the locale is en-US.
pub(crate) fn format_mb_date_short(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    // Month names are `mark`ed so the extractor registers the English literals;
    // they are translated once with `t(name)` at the format arms below.
    let month = |m: &str| -> Option<&'static str> {
        Some(match m {
            "01" => qbz_i18n::mark("January"),
            "02" => qbz_i18n::mark("February"),
            "03" => qbz_i18n::mark("March"),
            "04" => qbz_i18n::mark("April"),
            "05" => qbz_i18n::mark("May"),
            "06" => qbz_i18n::mark("June"),
            "07" => qbz_i18n::mark("July"),
            "08" => qbz_i18n::mark("August"),
            "09" => qbz_i18n::mark("September"),
            "10" => qbz_i18n::mark("October"),
            "11" => qbz_i18n::mark("November"),
            "12" => qbz_i18n::mark("December"),
            _ => return None,
        })
    };
    match parts.as_slice() {
        [y] => (*y).to_string(),
        [y, m] => match month(m) {
            // "{month} {year}" — translate the month name and the layout.
            Some(name) => {
                let name_tr = qbz_i18n::t(name);
                qbz_i18n::t_args("{} {}", &[name_tr.as_str(), *y])
            }
            None => date.to_string(),
        },
        [y, m, d] => match month(m) {
            Some(name) => {
                let day = d.trim_start_matches('0');
                let name_tr = qbz_i18n::t(name);
                // "{month} {day}, {year}" — translate name + layout.
                qbz_i18n::t_args("{} {}, {}", &[name_tr.as_str(), day, *y])
            }
            None => date.to_string(),
        },
        _ => date.to_string(),
    }
}

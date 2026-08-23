//! Small formatting helpers for the row builders.

/// Parse a stored folder color into a Slint color. Only solid `#rgb` /
/// `#rrggbb` hex is representable; gradients ("linear-gradient(...)") and
/// CSS vars ("var(--accent-primary)") and empty values return None so the
/// card falls back to the accent.
pub(super) fn parse_color(s: &str) -> Option<slint::Color> {
    let hex = s.strip_prefix('#')?;
    let (r, g, b) = match hex.len() {
        3 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            let r = ((v >> 8) & 0xf) as u8;
            let g = ((v >> 4) & 0xf) as u8;
            let b = (v & 0xf) as u8;
            (r * 17, g * 17, b * 17)
        }
        6 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            (((v >> 16) & 0xff) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
        }
        _ => return None,
    };
    Some(slint::Color::from_rgb_u8(r, g, b))
}

/// Total-playtime label, e.g. "1h 43m" or "12m" (mirrors Tauri's
/// `formatDuration`). Empty when the duration is zero.
pub(super) fn format_duration(seconds: u32) -> String {
    if seconds == 0 {
        return String::new();
    }
    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    if hours > 0 {
        qbz_i18n::t_args("{} h {} min", &[&hours.to_string(), &mins.to_string()])
    } else {
        qbz_i18n::t_args("{} min", &[&mins.to_string()])
    }
}

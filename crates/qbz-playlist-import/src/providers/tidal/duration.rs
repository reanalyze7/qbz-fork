//! ISO-8601 duration parsing — pure.

pub(super) fn parse_duration_ms(value: &str) -> Option<u64> {
    if !value.starts_with('P') {
        return None;
    }

    let mut seconds = 0u64;
    let mut parsed_any = false;
    let mut num = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            num.push(ch);
            continue;
        }

        if num.is_empty() {
            continue;
        }

        let value_num: u64 = num.parse().ok()?;
        parsed_any = true;
        match ch {
            'H' => seconds += value_num * 3600,
            'M' => seconds += value_num * 60,
            'S' => seconds += value_num,
            _ => {}
        }
        num.clear();
    }

    if !parsed_any {
        None
    } else {
        Some(seconds * 1000)
    }
}

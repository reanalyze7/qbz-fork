//! Pure `/proc/meminfo` field parsing.

mod detect;

pub(in crate::system_capabilities) use detect::detect_profile;
pub use detect::detect_profile_from_meminfo;

/// Parse the `MemTotal:` line out of `/proc/meminfo` content.
/// Returns None if the field is missing or unparseable.
pub fn parse_meminfo_total_kb(content: &str) -> Option<u64> {
    parse_meminfo_field_kb(content, "MemTotal:")
}

/// Parse the `MemAvailable:` line out of `/proc/meminfo` content.
/// Returns None if the field is missing or unparseable.
///
/// `MemAvailable` is the kernel's estimate of how much memory can be
/// allocated to a new workload without swapping — i.e. the right
/// metric for memory pressure (more accurate than `MemFree`, which
/// doesn't account for reclaimable page cache).
pub fn parse_meminfo_available_kb(content: &str) -> Option<u64> {
    parse_meminfo_field_kb(content, "MemAvailable:")
}

/// Shared parser for `<Field>: <number> kB` style /proc/meminfo lines.
fn parse_meminfo_field_kb(content: &str, field_prefix: &str) -> Option<u64> {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix(field_prefix) {
            let kb_str = rest.split_whitespace().next()?;
            return kb_str.parse::<u64>().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_meminfo_total_kb_extracts_value() {
        let sample = "\
MemTotal:         938196 kB
MemFree:          120000 kB
Buffers:           48000 kB
";
        assert_eq!(parse_meminfo_total_kb(sample), Some(938196));
    }

    #[test]
    fn parse_meminfo_total_kb_ignores_other_fields() {
        let sample = "\
MemFree:          120000 kB
MemTotal:        4194304 kB
SwapTotal:       2097152 kB
";
        assert_eq!(parse_meminfo_total_kb(sample), Some(4194304));
    }

    #[test]
    fn parse_meminfo_total_kb_handles_missing_field() {
        let sample = "\
MemFree:          120000 kB
SwapTotal:       2097152 kB
";
        assert_eq!(parse_meminfo_total_kb(sample), None);
    }

    #[test]
    fn parse_meminfo_total_kb_handles_empty_input() {
        assert_eq!(parse_meminfo_total_kb(""), None);
    }

    #[test]
    fn parse_meminfo_available_kb_extracts_value() {
        let sample = "\
MemTotal:         938196 kB
MemFree:          120000 kB
MemAvailable:     180000 kB
";
        assert_eq!(parse_meminfo_available_kb(sample), Some(180000));
    }

    #[test]
    fn parse_meminfo_available_kb_distinguishes_from_memtotal() {
        let sample = "\
MemAvailable:     500000 kB
MemTotal:        4194304 kB
";
        assert_eq!(parse_meminfo_available_kb(sample), Some(500000));
        assert_eq!(parse_meminfo_total_kb(sample), Some(4194304));
    }

    #[test]
    fn parse_meminfo_available_kb_handles_missing_field() {
        let sample = "MemTotal: 938196 kB\nMemFree: 100000 kB\n";
        assert_eq!(parse_meminfo_available_kb(sample), None);
    }
}

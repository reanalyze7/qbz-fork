//! Profile detection built on top of `/proc/meminfo` parsing.

use super::super::MemoryProfile;
use super::parse_meminfo_total_kb;

/// Pure detection given `/proc/meminfo` content. Falls back to Normal
/// when MemTotal is missing or unparseable so we never accidentally
/// throttle a system whose meminfo we couldn't read.
pub fn detect_profile_from_meminfo(content: &str) -> MemoryProfile {
    parse_meminfo_total_kb(content)
        .map(MemoryProfile::from_total_kb)
        .unwrap_or_else(|| MemoryProfile::from_total_kb(u64::MAX))
}

/// Read `/proc/meminfo` and derive the profile. Returns the Normal-fallback
/// profile on platforms without `/proc/meminfo` (macOS, Windows) or when
/// the file is unreadable for any reason.
pub(in crate::system_capabilities) fn detect_profile() -> MemoryProfile {
    match std::fs::read_to_string("/proc/meminfo") {
        Ok(content) => detect_profile_from_meminfo(&content),
        Err(_) => MemoryProfile::from_total_kb(u64::MAX),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::MemoryClass;

    #[test]
    fn detect_profile_from_meminfo_falls_back_to_normal_when_unparseable() {
        let profile = detect_profile_from_meminfo("garbage\nno memtotal here\n");
        assert_eq!(profile.class, MemoryClass::Normal);
    }

    #[test]
    fn detect_profile_from_meminfo_returns_low_memory_for_pi() {
        let pi_meminfo = "\
MemTotal:         938196 kB
MemFree:          250000 kB
";
        let profile = detect_profile_from_meminfo(pi_meminfo);
        assert_eq!(profile.class, MemoryClass::LowMemory);
        assert_eq!(profile.mem_total_kb, 938196);
    }
}

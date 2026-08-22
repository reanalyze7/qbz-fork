//! Memory pressure snapshot used by the runtime watchdog.

use super::meminfo::parse_meminfo_available_kb;
use super::memory_profile;

/// Snapshot of current memory pressure relative to the host's
/// MemoryProfile, used by the runtime memory watchdog to decide
/// whether to evict caches and abort prefetches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryPressure {
    pub mem_available_kb: u64,
    pub mem_total_kb: u64,
    /// Available memory as a percentage of total (0.0–100.0).
    pub available_pct: f64,
    /// True when available memory falls below 15 % of total.
    pub is_low: bool,
    /// True when available memory falls below 5 % of total. The
    /// watchdog should drop caches at this point.
    pub is_critical: bool,
}

/// Build a pressure snapshot from raw figures.
///
/// Pure for testing; the real entry point is [`read_memory_pressure`].
pub fn pressure_from_figures(mem_available_kb: u64, mem_total_kb: u64) -> MemoryPressure {
    let total = mem_total_kb.max(1);
    let pct = (mem_available_kb as f64 / total as f64) * 100.0;
    MemoryPressure {
        mem_available_kb,
        mem_total_kb,
        available_pct: pct,
        is_low: pct < 15.0,
        is_critical: pct < 5.0,
    }
}

/// Read /proc/meminfo and return a pressure snapshot relative to the
/// detected MemoryProfile. Returns None on platforms without
/// /proc/meminfo or when the file is unreadable.
pub fn read_memory_pressure() -> Option<MemoryPressure> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mem_available_kb = parse_meminfo_available_kb(&content)?;
    let profile = memory_profile();
    Some(pressure_from_figures(mem_available_kb, profile.mem_total_kb))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_healthy_machine_is_neither_low_nor_critical() {
        // 4 GB total, 2 GB available => 50%
        let p = pressure_from_figures(2 * 1024 * 1024, 4 * 1024 * 1024);
        assert!((p.available_pct - 50.0).abs() < 0.001);
        assert!(!p.is_low);
        assert!(!p.is_critical);
    }

    #[test]
    fn pressure_at_14pct_is_low_not_critical() {
        // Exactly 14 % available — under 15 % low threshold but well
        // above the 5 % critical floor.
        let total = 1_000_000;
        let avail = 140_000;
        let p = pressure_from_figures(avail, total);
        assert!(p.is_low);
        assert!(!p.is_critical);
    }

    #[test]
    fn pressure_at_4pct_is_both_low_and_critical() {
        let total = 1_000_000;
        let avail = 40_000;
        let p = pressure_from_figures(avail, total);
        assert!(p.is_low);
        assert!(p.is_critical);
    }

    #[test]
    fn pressure_threshold_boundaries_inclusive_above() {
        // 15.0 % should not register as low (strict inequality).
        let p = pressure_from_figures(150_000, 1_000_000);
        assert!(!p.is_low);
        // Just under: 14.999 % -> low.
        let p = pressure_from_figures(149_990, 1_000_000);
        assert!(p.is_low);

        // 5.0 % should not register as critical.
        let p = pressure_from_figures(50_000, 1_000_000);
        assert!(!p.is_critical);
    }

    #[test]
    fn pressure_handles_zero_total_safely() {
        // The .max(1) guard prevents divide-by-zero. The resulting pct
        // is meaningless but we don't panic.
        let p = pressure_from_figures(100, 0);
        assert!(p.available_pct.is_finite());
    }

    #[test]
    fn pi3b_critical_pressure_matches_codehd7_scenario() {
        // codehd7's reported case (issue #331 comments): Pi 3B with
        // ~938 MB total, swap thrash drives MemAvailable below 30 MB.
        // The watchdog should treat this as critical.
        let p = pressure_from_figures(25 * 1024, 938 * 1024);
        assert!(p.is_critical);
    }
}

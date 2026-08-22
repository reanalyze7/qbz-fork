//! Memory class classification and the derived runtime profile.

/// Memory class bucket the runtime adapts behavior to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryClass {
    /// >= 2 GB RAM. Default behavior, no caps applied.
    Normal,
    /// < 2 GB RAM. Reduces prefetch and buffer footprints to keep room
    /// for the WebView and avoid swap thrash on Raspberry Pi-class
    /// devices.
    LowMemory,
}

/// Derived runtime profile applied to memory-sensitive subsystems.
#[derive(Debug, Clone, Copy)]
pub struct MemoryProfile {
    pub class: MemoryClass,
    pub mem_total_kb: u64,
    /// How many upcoming Qobuz tracks to prefetch. Hi-Res tracks are
    /// ~60 MB each held in memory, so this is the dominant source of
    /// RSS growth during normal playback.
    pub prefetch_count: usize,
    /// Maximum allowed initial streaming buffer size in bytes. Caps the
    /// dynamic-buffer growth that `from_speed_mbps` would otherwise
    /// inflate to 2 MB on slow connections — exactly the wrong direction
    /// on a memory-pressured Pi where slow downloads are themselves a
    /// symptom of swap thrash.
    pub max_initial_buffer_bytes: usize,
    /// Concurrency cap for prefetch downloads.
    pub max_concurrent_prefetch: usize,
    /// When false, prefetch downgrades from HiRes/UltraHiRes to Lossless
    /// (44.1 kHz / 16-bit FLAC) so each cached track stays under ~15 MB
    /// instead of ~60 MB.
    pub allow_hires_prefetch: bool,
    /// Upper bound for the L1 (in-memory) audio cache. The default
    /// 400 MB cap is sized for Normal-class desktops; on a Pi 3B (1 GB
    /// total) that single subsystem could consume 40 % of RAM, which
    /// guarantees swap thrash before the watchdog can react.
    pub audio_cache_l1_max_bytes: usize,
}

impl MemoryProfile {
    /// Derive the profile from a total-memory figure (KB).
    pub(crate) fn from_total_kb(mem_total_kb: u64) -> Self {
        // Threshold: 2 GiB. Anything with at least 2 GB physical RAM is
        // assumed to have enough headroom for the WebView (~150 MB) plus
        // 5 cached HiRes tracks (~300 MB) plus typical app overhead.
        const NORMAL_FLOOR_KB: u64 = 2 * 1024 * 1024;

        if mem_total_kb >= NORMAL_FLOOR_KB {
            Self {
                class: MemoryClass::Normal,
                mem_total_kb,
                prefetch_count: 5,
                max_initial_buffer_bytes: 2 * 1024 * 1024,
                max_concurrent_prefetch: 2,
                allow_hires_prefetch: true,
                audio_cache_l1_max_bytes: 400 * 1024 * 1024,
            }
        } else {
            Self {
                class: MemoryClass::LowMemory,
                mem_total_kb,
                prefetch_count: 1,
                max_initial_buffer_bytes: 256 * 1024,
                max_concurrent_prefetch: 1,
                allow_hires_prefetch: false,
                audio_cache_l1_max_bytes: 50 * 1024 * 1024,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pi3b_with_1gb_resolves_to_low_memory() {
        // Raspberry Pi 3B = 1 GB RAM = ~938196 kB after kernel reservations.
        let profile = MemoryProfile::from_total_kb(938196);
        assert_eq!(profile.class, MemoryClass::LowMemory);
        assert_eq!(profile.prefetch_count, 1);
        assert_eq!(profile.max_concurrent_prefetch, 1);
        assert!(!profile.allow_hires_prefetch);
        assert!(profile.max_initial_buffer_bytes <= 256 * 1024);
        // L1 cap must be a small fraction of total RAM — at most ~10 %
        // of a Pi 3B, so we don't reserve four-tenths of memory for one
        // subsystem on a 1 GB host.
        assert!(profile.audio_cache_l1_max_bytes <= 100 * 1024 * 1024);
    }

    #[test]
    fn audio_cache_cap_is_significantly_smaller_on_low_memory() {
        let normal = MemoryProfile::from_total_kb(8 * 1024 * 1024);
        let low = MemoryProfile::from_total_kb(938196);
        assert!(low.audio_cache_l1_max_bytes < normal.audio_cache_l1_max_bytes);
    }

    #[test]
    fn pi_zero_2w_512mb_resolves_to_low_memory() {
        let profile = MemoryProfile::from_total_kb(500 * 1024);
        assert_eq!(profile.class, MemoryClass::LowMemory);
    }

    #[test]
    fn machine_with_2gb_resolves_to_normal() {
        // Exactly the threshold — Normal (>= NORMAL_FLOOR_KB).
        let profile = MemoryProfile::from_total_kb(2 * 1024 * 1024);
        assert_eq!(profile.class, MemoryClass::Normal);
        assert_eq!(profile.prefetch_count, 5);
        assert!(profile.allow_hires_prefetch);
    }

    #[test]
    fn machine_with_just_under_2gb_resolves_to_low_memory() {
        let profile = MemoryProfile::from_total_kb(2 * 1024 * 1024 - 1);
        assert_eq!(profile.class, MemoryClass::LowMemory);
    }
}

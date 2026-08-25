//! The "Hi-Res only" predicate for the playlist track list.
//!
//! Split into its own (pure) module so it is unit-testable: `refresh_view`
//! needs a live `AppWindow`, this does not. The tier strings come from
//! `TrackItem::quality_tier` — "hires" | "cd" | "mp3" | "lossless" | "".

/// Should a row with `tier` survive the filter?
///
/// Filter off -> everything survives. Filter on -> only 24-bit rows. Rows
/// whose tier is still unknown ("") are DROPPED when the filter is on: the
/// user asked to see Hi-Res, and "maybe" is not Hi-Res.
pub(super) fn keeps(tier: &str, hires_only: bool) -> bool {
    !hires_only || tier == "hires"
}

#[cfg(test)]
mod tests {
    use super::keeps;

    #[test]
    fn filter_off_keeps_every_tier() {
        for tier in ["hires", "lossless", "cd", "mp3", ""] {
            assert!(keeps(tier, false), "tier {tier:?} dropped with filter off");
        }
    }

    #[test]
    fn filter_on_keeps_only_hires() {
        assert!(keeps("hires", true));
        for tier in ["lossless", "cd", "mp3"] {
            assert!(!keeps(tier, true), "tier {tier:?} survived the filter");
        }
    }

    #[test]
    fn filter_on_drops_unknown_tier() {
        assert!(!keeps("", true));
    }

    #[test]
    fn tier_match_is_exact_not_prefix() {
        // Guards against a future `starts_with` "optimisation".
        assert!(!keeps("hires-lossless", true));
        assert!(!keeps("HIRES", true));
    }
}

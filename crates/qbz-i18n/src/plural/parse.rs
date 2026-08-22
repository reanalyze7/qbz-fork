//! Header-parsing internals for [`super::PluralRule::parse`]: extracting
//! `nplurals` and detecting which [`super::Kind`] a normalized header value
//! expresses.

use super::Kind;

/// Detect the plural-selection [`Kind`] from a whitespace-stripped header.
///
/// Match the Slavic 3-form rule by its distinctive `n%10==1&&n%100!=11`
/// signature, the single-form rule by `nplurals=1`/`plural=0`, then the two
/// 2-form rules; anything else defaults to the English `(n != 1)`.
pub(super) fn detect_kind(normalized: &str, nplurals: usize) -> Kind {
    if normalized.contains("n%10==1&&n%100!=11") {
        Kind::Russian
    } else if nplurals == 1 || normalized.contains("plural=0;") || normalized.contains("plural=0") {
        Kind::Single
    } else if normalized.contains("plural=(n>1)") || normalized.contains("plural=n>1") {
        Kind::GreaterThanOne
    } else {
        // Default and explicit `(n != 1)` both land here.
        Kind::NotOne
    }
}

/// Extract the integer after `nplurals=` from a whitespace-stripped header.
pub(super) fn parse_nplurals(normalized: &str) -> Option<usize> {
    let idx = normalized.find("nplurals=")? + "nplurals=".len();
    let rest = &normalized[idx..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

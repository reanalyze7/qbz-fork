//! Minimal gettext `Plural-Forms` evaluator.
//!
//! We support the expressions our locales actually use:
//!   - `nplurals=2; plural=(n != 1);`  (en, es, de, pt, nl)
//!   - `nplurals=2; plural=(n > 1);`   (fr)
//!   - `nplurals=1; plural=0;`         (ja — no plural distinction)
//!   - `nplurals=3; plural=(n%10==1 && n%100!=11 ? 0 : n%10>=2 && n%10<=4 &&
//!      (n%100<12 || n%100>14) ? 1 : 2);` (ru — Slavic one/few/many)
//! Anything unrecognized falls back to the English default `if n==1 {0} else {1}`.

mod parse;

#[cfg(test)]
mod tests;

use parse::detect_kind;

/// The plural-selection kind extracted from a `Plural-Forms` header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// `plural=(n != 1)` — index 0 when n == 1, else 1.
    NotOne,
    /// `plural=(n > 1)` — index 0 when n <= 1, else 1.
    GreaterThanOne,
    /// `nplurals=1; plural=0` — a single form for every count (e.g. Japanese).
    Single,
    /// Slavic three-form rule (e.g. Russian): one / few / many.
    Russian,
}

/// A parsed gettext plural rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluralRule {
    nplurals: usize,
    kind: Kind,
}

impl Default for PluralRule {
    fn default() -> Self {
        PluralRule {
            nplurals: 2,
            kind: Kind::NotOne,
        }
    }
}

impl PluralRule {
    /// Parse a `Plural-Forms` header value (the part after `Plural-Forms:`).
    ///
    /// Accepts the full header line or just the value; tolerant of whitespace.
    /// Unknown plural expressions default to `(n != 1)` with `nplurals=2`.
    pub fn parse(plural_forms_header: &str) -> PluralRule {
        // Normalize: drop spaces so `n != 1` and `n!=1` both match.
        let normalized: String = plural_forms_header
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        let nplurals = parse::parse_nplurals(&normalized).unwrap_or(2);
        let kind = detect_kind(&normalized, nplurals);

        PluralRule { nplurals, kind }
    }

    /// Number of plural forms (`nplurals`).
    pub fn nplurals(&self) -> usize {
        self.nplurals
    }

    /// Index of the plural form to use for count `n`.
    pub fn index(&self, n: i64) -> usize {
        match self.kind {
            Kind::NotOne => {
                if n == 1 {
                    0
                } else {
                    1
                }
            }
            Kind::GreaterThanOne => {
                if n > 1 {
                    1
                } else {
                    0
                }
            }
            // One form for every count (Japanese): always index 0.
            Kind::Single => 0,
            // Slavic one/few/many (Russian). Counts are non-negative here; guard
            // against negatives so the modulo arithmetic stays well-defined.
            Kind::Russian => {
                let n = n.unsigned_abs();
                if n % 10 == 1 && n % 100 != 11 {
                    0
                } else if (2..=4).contains(&(n % 10)) && !(12..=14).contains(&(n % 100)) {
                    1
                } else {
                    2
                }
            }
        }
    }
}

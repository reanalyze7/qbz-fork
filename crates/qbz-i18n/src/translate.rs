//! Translation lookup functions: `mark`/`t`/`tn`/`t_args`/`tf` and the
//! `{}`-placeholder substitution they share.

use crate::current_catalog;

/// Marks a string literal for catalog extraction at its DEFINITION site without
/// translating here. Pair with a later `t(value)` that does the lookup. The
/// extractor scans `mark("...")`; runtime returns the literal unchanged.
pub const fn mark(s: &'static str) -> &'static str {
    s
}

/// Translate `msgid` (singular) in the current language.
/// Falls back to the English `msgid` itself when untranslated.
pub fn t(msgid: &str) -> String {
    current_catalog()
        .get(msgid)
        .map(|s| s.to_string())
        .unwrap_or_else(|| msgid.to_string())
}

/// Translate a plural form for count `n` in the current language.
/// Falls back to the English `singular`/`plural` (`if n==1`) when untranslated.
pub fn tn(singular: &str, plural: &str, n: i64) -> String {
    let cat = current_catalog();
    let form = cat.plural_rule().index(n);
    if let Some(translated) = cat.get_plural(singular, form) {
        return translated.to_string();
    }
    if n == 1 {
        singular.to_string()
    } else {
        plural.to_string()
    }
}

/// [`t`] then substitute `{}` placeholders left-to-right with `args`.
pub fn t_args(msgid: &str, args: &[&str]) -> String {
    substitute(&t(msgid), args)
}

/// [`tn`] then substitute `{}` placeholders left-to-right with `args`.
pub fn tf(singular: &str, plural: &str, n: i64, args: &[&str]) -> String {
    substitute(&tn(singular, plural, n), args)
}

/// Replace each `{}` with the next arg, left-to-right. Extra `{}` or extra
/// args are left untouched / ignored respectively.
pub(crate) fn substitute(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut args_iter = args.iter();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
            chars.next(); // consume '}'
            match args_iter.next() {
                Some(arg) => out.push_str(arg),
                None => out.push_str("{}"),
            }
        } else {
            out.push(c);
        }
    }
    out
}

//! Pure title-cleaning helper used to broaden Qobuz search attempts.

/// Remove parenthetical/bracket suffixes from a title.
/// "Senjutsu (2021 Remaster)" → "Senjutsu"
/// "The Number of the Beast [Deluxe Edition]" → "The Number of the Beast"
pub(crate) fn clean_title(title: &str) -> String {
    let mut result = title.to_string();
    // Remove trailing (...) and [...]
    while let Some(pos) = result.rfind('(') {
        if result[pos..].contains(')') {
            result = result[..pos].trim_end().to_string();
        } else {
            break;
        }
    }
    while let Some(pos) = result.rfind('[') {
        if result[pos..].contains(']') {
            result = result[..pos].trim_end().to_string();
        } else {
            break;
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_parens_and_brackets() {
        assert_eq!(clean_title("Senjutsu (2021 Remaster)"), "Senjutsu");
        assert_eq!(
            clean_title("The Number of the Beast [Deluxe Edition]"),
            "The Number of the Beast"
        );
        assert_eq!(clean_title("Plain Title"), "Plain Title");
    }
}

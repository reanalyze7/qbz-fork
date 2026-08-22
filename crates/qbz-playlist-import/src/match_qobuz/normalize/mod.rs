//! Pure string-normalization helpers used by the scorer.

#[cfg(test)]
mod tests;

pub(super) fn similarity(a: &str, b: &str) -> f32 {
    let na = normalize(a);
    let nb = normalize(b);

    if na.is_empty() || nb.is_empty() {
        return 0.0;
    }

    if na == nb {
        return 1.0;
    }

    if na.contains(&nb) || nb.contains(&na) {
        return 0.85;
    }

    token_overlap(&na, &nb)
}

fn normalize(input: &str) -> String {
    let stripped = remove_bracketed(input);
    let mut cleaned = String::new();

    for ch in stripped.chars() {
        if ch.is_ascii_alphanumeric() || ch.is_whitespace() {
            cleaned.push(ch.to_ascii_lowercase());
        } else {
            cleaned.push(' ');
        }
    }

    let stop_words = [
        "remaster",
        "remastered",
        "deluxe",
        "edition",
        "live",
        "feat",
        "featuring",
        "version",
        "mix",
        "mono",
        "stereo",
        "edit",
    ];

    cleaned
        .split_whitespace()
        .filter(|token| !stop_words.contains(token))
        .collect::<Vec<_>>()
        .join(" ")
}

fn remove_bracketed(input: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;

    for ch in input.chars() {
        match ch {
            '(' | '[' => {
                depth += 1;
            }
            ')' | ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {
                if depth == 0 {
                    out.push(ch);
                }
            }
        }
    }

    out
}

fn token_overlap(a: &str, b: &str) -> f32 {
    let a_tokens: Vec<&str> = a.split_whitespace().collect();
    let b_tokens: Vec<&str> = b.split_whitespace().collect();

    if a_tokens.is_empty() || b_tokens.is_empty() {
        return 0.0;
    }

    let mut matches = 0u32;
    for token in &a_tokens {
        if b_tokens.contains(token) {
            matches += 1;
        }
    }

    matches as f32 / a_tokens.len().max(b_tokens.len()) as f32
}

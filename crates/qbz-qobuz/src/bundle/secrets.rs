use crate::error::{ApiError, Result};
use regex::Regex;

pub(crate) fn extract_secrets(bundle: &str) -> Result<Vec<String>> {
    // Extract seeds with their timezone keys
    // Pattern: X.initialSeed("SEED",window.utimezone.TIMEZONE)
    let seed_re = Regex::new(
        r#"[a-z]\.initialSeed\("(?P<seed>[\w=]+)",window\.utimezone\.(?P<timezone>[a-z]+)\)"#,
    )
    .expect("Invalid regex");

    let mut seeds: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut timezones: Vec<String> = Vec::new();

    for caps in seed_re.captures_iter(bundle) {
        if let (Some(seed), Some(tz)) = (caps.name("seed"), caps.name("timezone")) {
            let tz_str = tz.as_str().to_string();
            seeds.insert(tz_str.clone(), seed.as_str().to_string());
            timezones.push(tz_str);
        }
    }

    log::debug!(
        "Found {} seeds with timezones: {:?}",
        seeds.len(),
        timezones
    );

    if seeds.is_empty() {
        return Err(ApiError::BundleExtractionError(
            "No seeds found".to_string(),
        ));
    }

    // Build dynamic regex with found timezones (capitalize first letter for matching)
    // Pattern: name:"\w+/Timezone",info:"INFO",extras:"EXTRAS"
    let tz_pattern: Vec<String> = timezones
        .iter()
        .map(|tz| {
            // Capitalize first letter: "berlin" -> "Berlin"
            let mut chars = tz.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect();

    let tz_alternatives = tz_pattern.join("|");
    let info_pattern = format!(
        r#"name:"\w+/(?P<timezone>{})",info:"(?P<info>[\w=]+)",extras:"(?P<extras>[\w=]+)""#,
        tz_alternatives
    );

    log::debug!("Info regex pattern: {}", info_pattern);

    let info_re = Regex::new(&info_pattern).expect("Invalid info regex");

    let mut secrets = Vec::new();

    for caps in info_re.captures_iter(bundle) {
        if let (Some(tz), Some(info), Some(extras)) = (
            caps.name("timezone"),
            caps.name("info"),
            caps.name("extras"),
        ) {
            // Convert capitalized timezone back to lowercase for lookup
            let tz_lower = tz.as_str().to_lowercase();
            if let Some(seed) = seeds.get(&tz_lower) {
                // Concatenate seed + info + extras, remove last 44 chars, base64 decode
                let combined = format!("{}{}{}", seed, info.as_str(), extras.as_str());
                log::debug!(
                    "Combined length: {}, timezone: {}",
                    combined.len(),
                    tz_lower
                );

                if combined.len() > 44 {
                    let trimmed = &combined[..combined.len() - 44];
                    match base64::Engine::decode(
                        &base64::engine::general_purpose::STANDARD,
                        trimmed,
                    ) {
                        Ok(decoded) => {
                            if let Ok(secret) = String::from_utf8(decoded) {
                                log::info!(
                                    "Successfully extracted secret for timezone: {}",
                                    tz_lower
                                );
                                secrets.push(secret);
                            }
                        }
                        Err(e) => {
                            log::debug!("Base64 decode failed for {}: {}", tz_lower, e);
                        }
                    }
                }
            }
        }
    }

    // If the complex extraction fails, try a simpler pattern
    // that might work for some bundle versions
    if secrets.is_empty() {
        log::warn!("Complex extraction failed, trying simple appSecret pattern");
        let simple_re = Regex::new(r#"appSecret:"([a-f0-9]{32})""#).expect("Invalid regex");
        for caps in simple_re.captures_iter(bundle) {
            if let Some(secret) = caps.get(1) {
                secrets.push(secret.as_str().to_string());
            }
        }
    }

    log::info!("Extracted app secrets from bundle");
    Ok(secrets)
}

use crate::error::{ApiError, Result};
use regex::Regex;

pub(crate) use super::secrets::extract_secrets;

pub(crate) fn extract_bundle_url(html: &str) -> Result<String> {
    // Pattern: <script src="/resources/X.X.X-bXXX/bundle.js"></script>
    let re =
        Regex::new(r#"<script src="(/resources/\d+\.\d+\.\d+-[a-z]\d{3}/bundle\.js)"></script>"#)
            .expect("Invalid regex");

    re.captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| ApiError::BundleExtractionError("Bundle URL not found".to_string()))
}

pub(crate) fn extract_app_id(bundle: &str) -> Result<String> {
    // Pattern: production:{api:{appId:"XXXXXXXXX"
    let re = Regex::new(r#"production:\{api:\{appId:"(?P<app_id>\d{9})""#).expect("Invalid regex");

    re.captures(bundle)
        .and_then(|caps| caps.name("app_id"))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| ApiError::BundleExtractionError("App ID not found".to_string()))
}

pub(crate) fn extract_private_key(bundle: &str) -> Option<String> {
    // Pattern: privateKey:"VALUE" (the static OAuth key used in /oauth/callback)
    let re = Regex::new(r#"privateKey:\s*"(?P<key>[A-Za-z0-9]{6,30})""#).expect("Invalid regex");

    re.captures(bundle)
        .and_then(|caps| caps.name("key"))
        .map(|m| m.as_str().to_string())
}

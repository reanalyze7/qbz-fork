use chrono::{TimeZone, Utc};

use crate::error::{ApiError, Result};
use qbz_models::UserSession;

fn parse_subscription_valid_until(parameters: &serde_json::Value) -> Option<String> {
    // Try common string fields first.
    let string_keys = [
        "end_date",
        "expiration_date",
        "valid_until",
        "expires_at",
        "expiry_date",
    ];
    for key in string_keys {
        if let Some(s) = parameters.get(key).and_then(|v| v.as_str()) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Try common timestamp fields (seconds).
    let ts_keys = [
        "end_date_ts",
        "expires_at_ts",
        "expiration_ts",
        "valid_until_ts",
    ];
    for key in ts_keys {
        if let Some(ts) = parameters.get(key).and_then(|v| v.as_i64()) {
            if ts > 0 {
                return Some(Utc.timestamp_opt(ts, 0).single()?.date_naive().to_string());
            }
        }
    }

    None
}

/// Parse user login response
pub fn parse_login_response(response: &serde_json::Value) -> Result<UserSession> {
    let user = response
        .get("user")
        .ok_or_else(|| ApiError::AuthenticationError("No user in response".to_string()))?;

    let user_auth_token = response
        .get("user_auth_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::AuthenticationError("No auth token in response".to_string()))?
        .to_string();

    let user_id = user
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ApiError::AuthenticationError("No user id".to_string()))?;

    let email = user
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let display_name = user
        .get("display_name")
        .and_then(|v| v.as_str())
        .or_else(|| user.get("login").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();

    // Check subscription
    let credential = user.get("credential");
    let subscription_label = credential
        .and_then(|c| c.get("parameters"))
        .and_then(|p| p.get("short_label"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let subscription_valid_until = credential
        .and_then(|c| c.get("parameters"))
        .and_then(parse_subscription_valid_until);

    // Account territory + language (snake_case wire names, verbatim in
    // Qobuz's own embedded /user/login fixture — see qbz-nix-docs
    // offline-mode/tauri-review-2026-06-09/10-subscription-trial-offline-
    // gating.md §1.2). Absent on older captures -> None (feature stays off).
    let country_code = user
        .get("country_code")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let language_code = user
        .get("language_code")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    // Check if user has valid subscription
    let has_subscription = credential
        .and_then(|c| c.get("parameters"))
        .map(|p| !p.is_null() && p.as_object().map(|o| !o.is_empty()).unwrap_or(false))
        .unwrap_or(false);

    if !has_subscription {
        return Err(ApiError::IneligibleUser);
    }

    Ok(UserSession {
        user_auth_token,
        user_id,
        email,
        display_name,
        subscription_label,
        subscription_valid_until,
        country_code,
        language_code,
    })
}

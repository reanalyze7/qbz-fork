//! User session / credentials.

use serde::{Deserialize, Serialize};

/// User credentials and session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_auth_token: String,
    pub user_id: u64,
    pub email: String,
    pub display_name: String,
    pub subscription_label: String,
    #[serde(default)]
    pub subscription_valid_until: Option<String>,
    /// Account territory (ISO 3166-1 alpha-2, e.g. "FR") from the login
    /// response. `serde(default)` keeps pre-v10 persisted sessions loadable.
    #[serde(default)]
    pub country_code: Option<String>,
    /// Account language (ISO 639-1, e.g. "fr") from the login response —
    /// the default target for lyrics translation ("Auto").
    #[serde(default)]
    pub language_code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_session_deserializes_pre_v10_json() {
        // Sessions persisted before the country/language capture must still
        // load: both new fields default to None (feature stays Auto-off).
        let json = r#"{
            "user_auth_token": "token",
            "user_id": 1705826,
            "email": "a@b.c",
            "display_name": "Tester",
            "subscription_label": "Studio",
            "subscription_valid_until": null
        }"#;
        let session: UserSession = serde_json::from_str(json).expect("old session json loads");
        assert_eq!(session.country_code, None);
        assert_eq!(session.language_code, None);
        assert_eq!(session.user_id, 1705826);
    }

    #[test]
    fn user_session_round_trips_country_and_language() {
        let json = r#"{
            "user_auth_token": "token",
            "user_id": 1705826,
            "email": "a@b.c",
            "display_name": "Tester",
            "subscription_label": "Studio",
            "subscription_valid_until": null,
            "country_code": "FR",
            "language_code": "fr"
        }"#;
        let session: UserSession = serde_json::from_str(json).expect("v10 session json loads");
        assert_eq!(session.country_code.as_deref(), Some("FR"));
        assert_eq!(session.language_code.as_deref(), Some("fr"));
        let back: UserSession =
            serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
        assert_eq!(back.language_code.as_deref(), Some("fr"));
    }
}

use super::QobuzClient;
use crate::auth::{get_timestamp, sign_request};
use crate::error::{ApiError, Result};

impl QobuzClient {
    // === Header helpers ===

    /// Build standard API headers.
    /// Always includes X-App-Id. Includes X-User-Auth-Token when logged in.
    pub(super) async fn api_headers(&self) -> Result<reqwest::header::HeaderMap> {
        use reqwest::header::{HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();

        let app_id = self.app_id().await?;
        headers.insert(
            "X-App-Id",
            HeaderValue::from_str(&app_id).map_err(|_| ApiError::InvalidAppId)?,
        );

        if let Ok(token) = self.auth_token().await {
            if let Ok(val) = HeaderValue::from_str(&token) {
                headers.insert("X-User-Auth-Token", val);
            }
        }

        Ok(headers)
    }

    /// Build headers that REQUIRE authentication. Fails if not logged in.
    pub(crate) async fn authenticated_headers(&self) -> Result<reqwest::header::HeaderMap> {
        use reqwest::header::{HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();

        let app_id = self.app_id().await?;
        headers.insert(
            "X-App-Id",
            HeaderValue::from_str(&app_id).map_err(|_| ApiError::InvalidAppId)?,
        );

        let token = self.auth_token().await?;
        headers.insert(
            "X-User-Auth-Token",
            HeaderValue::from_str(&token)
                .map_err(|_| ApiError::AuthenticationError("Invalid auth token format".into()))?,
        );

        Ok(headers)
    }

    /// Build a signed GET request. Computes request_sig from the endpoint method name
    /// and query params, then appends request_ts + request_sig to the query.
    /// `method_name` is the endpoint path without slashes, e.g. "albumget".
    pub(super) async fn signed_get(
        &self,
        url: &str,
        method_name: &str,
        params: &[(&str, String)],
    ) -> Result<reqwest::Response> {
        let timestamp = get_timestamp();
        let secret = self.secret().await?;
        let kv: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let sig = sign_request(method_name, &kv, timestamp, &secret);
        let ts_str = timestamp.to_string();

        let mut query_params: Vec<(&str, &str)> = kv;
        query_params.push(("request_ts", &ts_str));
        query_params.push(("request_sig", &sig));

        let response = self
            .http()?
            .get(url)
            .headers(self.api_headers().await?)
            .query(&query_params)
            .send()
            .await?;
        Ok(response)
    }

    /// Same as signed_get but uses authenticated headers (requires login).
    pub(super) async fn signed_get_auth(
        &self,
        url: &str,
        method_name: &str,
        params: &[(&str, String)],
    ) -> Result<reqwest::Response> {
        let timestamp = get_timestamp();
        let secret = self.secret().await?;
        let kv: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let sig = sign_request(method_name, &kv, timestamp, &secret);
        let ts_str = timestamp.to_string();

        let mut query_params: Vec<(&str, &str)> = kv;
        query_params.push(("request_ts", &ts_str));
        query_params.push(("request_sig", &sig));

        let response = self
            .http()?
            .get(url)
            .headers(self.authenticated_headers().await?)
            .query(&query_params)
            .send()
            .await?;
        Ok(response)
    }
}

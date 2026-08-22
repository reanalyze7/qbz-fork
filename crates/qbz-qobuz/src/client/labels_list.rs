use serde_json::Value;

use super::QobuzClient;
use crate::auth::{get_timestamp, sign_request};
use crate::endpoints::{self, paths};
use crate::error::Result;
use qbz_models::*;

impl QobuzClient {
    /// Bulk lookup a set of labels by ID (POST).
    ///
    /// Follows the same signing convention as `get_tracks_batch` (see
    /// `track/getList`): the sig covers the joined ID list as a query
    /// string key, and the JSON body carries the list itself.
    pub async fn get_label_list(&self, label_ids: &[u64]) -> Result<LabelGetListResponse> {
        let url = endpoints::build_url(paths::LABEL_GET_LIST);
        let headers = self.api_headers().await?;
        let timestamp = get_timestamp();
        let secret = self.secret().await?;
        let ids_str: String = label_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let sig = sign_request("labelgetList", &[("label_ids", &ids_str)], timestamp, &secret);

        let body = serde_json::json!({ "label_ids": label_ids });
        log::debug!("[API] get_label_list POST ({} ids)", label_ids.len());

        let response: Value = self
            .http()?
            .post(&url)
            .headers(headers)
            .query(&[("request_ts", timestamp.to_string()), ("request_sig", sig)])
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(serde_json::from_value(response)?)
    }
}

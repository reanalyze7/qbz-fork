//! Recording metadata hydration endpoint.

use super::{ListenBrainzClient, LISTENBRAINZ_API_URL};
use crate::error::{IntegrationError, IntegrationResult};
use crate::listenbrainz::models::*;

impl ListenBrainzClient {
    /// Hydrate CF `recording_mbid`s into names + artist mbids + cover art.
    ///
    /// `GET /metadata/recording/?recording_mbids={comma-joined}&inc=artist+release`
    ///
    /// The response is a JSON OBJECT keyed by `recording_mbid`:
    /// ```text
    /// { "<mbid>": { "recording": {"name": ...},
    ///               "artist": {"name": ..., "artists": [{"artist_mbid": ...}, ...]},
    ///               "release": {"name": ..., "caa_id": ..., "caa_release_mbid": ...} } }
    /// ```
    /// Iterates the object entries; the KEY is the `recording_mbid`. Empty input
    /// short-circuits to `Ok(vec![])`. HTTP 204/404 and empty bodies are also
    /// treated as "no data".
    pub async fn get_metadata_recordings(
        &self,
        recording_mbids: &[String],
    ) -> IntegrationResult<Vec<LbRecordingMeta>> {
        if recording_mbids.is_empty() {
            return Ok(vec![]);
        }

        let token = self.config.lock().await.token.clone();

        let url = format!("{}/metadata/recording/", LISTENBRAINZ_API_URL);
        let joined = recording_mbids.join(",");

        // `inc=artist release` is form-encoded to `inc=artist+release` by reqwest,
        // which is exactly the value ListenBrainz expects.
        let mut request = self.client.get(&url).query(&[
            ("recording_mbids", joined.as_str()),
            ("inc", "artist release"),
        ]);
        if let Some(token) = token {
            request = request.header("Authorization", format!("Token {}", token));
        }

        let response = request.send().await?;
        let status = response.status();

        if status == reqwest::StatusCode::NO_CONTENT || status == reqwest::StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(IntegrationError::internal(format!(
                "ListenBrainz metadata recordings failed: {} - {}",
                status, text
            )));
        }

        let body = response.text().await.unwrap_or_default();
        if body.trim().is_empty() {
            return Ok(vec![]);
        }
        let json: serde_json::Value = match serde_json::from_str(&body) {
            Ok(value) => value,
            Err(_) => return Ok(vec![]),
        };

        let object = match json.as_object() {
            Some(object) => object,
            None => return Ok(vec![]),
        };

        let mut recordings = Vec::with_capacity(object.len());
        for (recording_mbid, entry) in object {
            let recording_name = entry
                .get("recording")
                .and_then(|recording| recording.get("name"))
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();

            let artist = entry.get("artist");
            let artist_name = artist
                .and_then(|artist| artist.get("name"))
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string();
            let artist_mbids = artist
                .and_then(|artist| artist.get("artists"))
                .and_then(|artists| artists.as_array())
                .map(|artists| {
                    artists
                        .iter()
                        .filter_map(|credit| {
                            credit
                                .get("artist_mbid")
                                .and_then(|value| value.as_str())
                                .map(|value| value.to_string())
                        })
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            let release = entry.get("release");
            let release_name = release
                .and_then(|release| release.get("name"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let caa_id = release
                .and_then(|release| release.get("caa_id"))
                .and_then(|value| value.as_i64());
            let caa_release_mbid = release
                .and_then(|release| release.get("caa_release_mbid"))
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());

            recordings.push(LbRecordingMeta {
                recording_mbid: recording_mbid.clone(),
                recording_name,
                artist_name,
                artist_mbids,
                release_name,
                caa_id,
                caa_release_mbid,
            });
        }

        Ok(recordings)
    }
}

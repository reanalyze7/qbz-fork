//! `StreamFetcher::fetch_to_memory` — to-memory download (no progress events).

use super::StreamFetcher;

impl StreamFetcher {
    /// Fetch to memory (for smaller files or streaming)
    pub async fn fetch_to_memory(&self, url: &str) -> Result<Vec<u8>, String> {
        let client = Self::build_client()?;

        let response = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read bytes: {}", e))?;

        Ok(bytes.to_vec())
    }
}

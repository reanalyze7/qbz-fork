//! GitHub release fetch + parse for the running version's release notes.

use serde::Deserialize;

use super::GITHUB_RELEASES_URL;

/// GitHub release JSON (only the fields the modal needs).
#[derive(Debug, Clone, Deserialize)]
struct GithubRelease {
    tag_name: String,
    published_at: String,
    body: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// The parsed release the controller applies to the UI.
pub(super) struct FetchedRelease {
    pub version: String,
    pub date: String,
    pub body: Option<String>,
}

/// Fetch the release for `version` by exact tag (`v{version}`), with the
/// GitHub-required `User-Agent`. Returns `None` on any network/parse failure or
/// for draft/prerelease tags (silent — the modal shows its empty state).
pub(super) async fn fetch_release_for_version(version: &str) -> Option<FetchedRelease> {
    let tag = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    };
    let url = format!("{GITHUB_RELEASES_URL}/tags/{tag}");

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("qbz")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[qbz-slint] whats-new client build failed: {e}");
            return None;
        }
    };

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[qbz-slint] whats-new fetch failed for {tag}: {e}");
            return None;
        }
    };
    if !resp.status().is_success() {
        log::warn!("[qbz-slint] whats-new fetch HTTP {} for {tag}", resp.status());
        return None;
    }

    let release: GithubRelease = match resp.json().await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[qbz-slint] whats-new JSON parse failed for {tag}: {e}");
            return None;
        }
    };
    if release.draft || release.prerelease {
        return None;
    }

    Some(FetchedRelease {
        version: normalize_version_tag(&release.tag_name),
        date: format_release_date(&release.published_at),
        body: release.body,
    })
}

fn normalize_version_tag(tag: &str) -> String {
    tag.trim().trim_start_matches('v').to_string()
}

/// Format an RFC3339 timestamp as "Mon D, YYYY" (en-US short), mirroring the
/// Tauri `formatReleaseDate`. Falls back to the raw string on parse failure.
fn format_release_date(iso: &str) -> String {
    use chrono::{DateTime, Datelike};
    let Ok(dt) = DateTime::parse_from_rfc3339(iso) else {
        return iso.to_string();
    };
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let m = dt.month0() as usize;
    let month = MONTHS.get(m).copied().unwrap_or("");
    format!("{} {}, {}", month, dt.day(), dt.year())
}

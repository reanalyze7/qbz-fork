use super::types::ProbeOutcome;
use std::time::Duration;

/// How a single probe endpoint decides success. Strict on purpose: Tauri
/// accepted any 2xx/3xx, which read captive portals as online.
pub(super) enum ProbeExpect {
    /// Exactly HTTP 204, empty-ish body (generate_204 contract).
    Status204,
    /// HTTP 200 and the body contains the marker.
    BodyContains(&'static str),
}

pub(super) struct ProbeEndpoint {
    url: &'static str,
    expect: ProbeExpect,
}

/// Vendor-diverse probe set. The first entry is IP-literal: it works with
/// DNS completely broken, which was the most likely residual false-offline
/// vector after #467 (all three old endpoints were hostname-based).
pub(super) const PROBES: &[ProbeEndpoint] = &[
    ProbeEndpoint {
        url: "https://1.1.1.1/cdn-cgi/trace",
        expect: ProbeExpect::BodyContains("ip="),
    },
    ProbeEndpoint {
        url: "https://connectivitycheck.gstatic.com/generate_204",
        expect: ProbeExpect::Status204,
    },
    ProbeEndpoint {
        url: "https://www.msftconnecttest.com/connecttest.txt",
        expect: ProbeExpect::BodyContains("Microsoft Connect Test"),
    },
];

pub(super) const PROBE_TIMEOUT: Duration = Duration::from_secs(8);

async fn probe_endpoint(client: &reqwest::Client, ep: &ProbeEndpoint) -> ProbeOutcome {
    match client.get(ep.url).send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_redirection() {
                return ProbeOutcome::CaptivePortal;
            }
            match ep.expect {
                ProbeExpect::Status204 => {
                    if status.as_u16() == 204 {
                        ProbeOutcome::Success
                    } else {
                        ProbeOutcome::Failure
                    }
                }
                ProbeExpect::BodyContains(marker) => {
                    if status.as_u16() != 200 {
                        return ProbeOutcome::Failure;
                    }
                    match response.text().await {
                        Ok(body) if body.contains(marker) => ProbeOutcome::Success,
                        _ => ProbeOutcome::Failure,
                    }
                }
            }
        }
        Err(_) => ProbeOutcome::Failure,
    }
}

/// Race the probe set; first validated success wins. Redirect answers are
/// remembered: all-fail + any-redirect = captive portal.
pub async fn probe_all(client: &reqwest::Client) -> ProbeOutcome {
    let mut set = tokio::task::JoinSet::new();
    for ep in PROBES {
        let client = client.clone();
        set.spawn(async move { probe_endpoint(&client, ep).await });
    }

    let mut saw_captive = false;
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(ProbeOutcome::Success) => {
                set.abort_all();
                return ProbeOutcome::Success;
            }
            Ok(ProbeOutcome::CaptivePortal) => saw_captive = true,
            _ => {}
        }
    }
    if saw_captive {
        ProbeOutcome::CaptivePortal
    } else {
        ProbeOutcome::Failure
    }
}

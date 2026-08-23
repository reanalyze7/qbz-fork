// ===================== OS route signal (Linux) =====================

/// Parse `/proc/net/route` content: any non-loopback entry with destination
/// 00000000 is an IPv4 default route.
pub(super) fn ipv4_has_default_route(content: &str) -> bool {
    content.lines().skip(1).any(|line| {
        let mut cols = line.split_whitespace();
        let iface = cols.next().unwrap_or("");
        let dest = cols.next().unwrap_or("");
        iface != "lo" && dest == "00000000"
    })
}

/// Parse `/proc/net/ipv6_route` content: any non-loopback entry with
/// destination ::/0 (32 zero hex chars, prefix length 00) is a default route.
pub(super) fn ipv6_has_default_route(content: &str) -> bool {
    content.lines().any(|line| {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 10 {
            return false;
        }
        let dest = cols[0];
        let prefix_len = cols[1];
        let iface = cols[9];
        iface != "lo"
            && prefix_len == "00"
            && dest.len() == 32
            && dest.bytes().all(|b| b == b'0')
    })
}

/// `Some(true)` = at least one default route exists; `Some(false)` = readable
/// and definitely none; `None` = signal unavailable (non-Linux or read error)
/// — the caller falls back to probes only.
pub fn has_default_route() -> Option<bool> {
    #[cfg(target_os = "linux")]
    {
        let v4 = std::fs::read_to_string("/proc/net/route")
            .map(|c| ipv4_has_default_route(&c))
            .ok();
        let v6 = std::fs::read_to_string("/proc/net/ipv6_route")
            .map(|c| ipv6_has_default_route(&c))
            .ok();
        match (v4, v6) {
            (None, None) => None,
            (a, b) => Some(a.unwrap_or(false) || b.unwrap_or(false)),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

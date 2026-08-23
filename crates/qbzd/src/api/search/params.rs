/// Default result count per category when the caller gives no `limit`.
pub(super) const DEFAULT_LIMIT: u32 = 20;
/// Upper bound on `limit` — a control-plane search is top-hits, not a pager
/// (deep paging belongs in the GUI). Silently clamped, never a 400.
pub(super) const MAX_LIMIT: u32 = 100;

/// The four searchable categories (`type=<one>`), plus the `all` fan-out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Category {
    Albums,
    Tracks,
    Artists,
    Playlists,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SearchType {
    All,
    One(Category),
}

impl SearchType {
    /// The canonical `type` string echoed back in the response (matches the CLI
    /// flag values 1:1 so a script round-trips `--type` through `--json`).
    pub(super) fn as_str(self) -> &'static str {
        match self {
            SearchType::All => "all",
            SearchType::One(Category::Albums) => "albums",
            SearchType::One(Category::Tracks) => "tracks",
            SearchType::One(Category::Artists) => "artists",
            SearchType::One(Category::Playlists) => "playlists",
        }
    }
}

#[derive(Debug)]
pub(super) struct SearchParams {
    pub(super) q: String,
    pub(super) stype: SearchType,
    pub(super) limit: u32,
    pub(super) offset: u32,
}

/// Parse `q`/`query` (percent-decoded), `type` (strict literal, default `all`),
/// `limit` (clamped 1..=MAX, default 20), `offset` (default 0). A missing/blank
/// query is a 400 (§1.4 error voice); an unknown `type` is a 400; malformed
/// numeric params degrade to defaults (a read route never 400s on a bad number,
/// mirroring `queue::parse_offset_limit`).
pub(super) fn parse_query(query: &str) -> Result<SearchParams, (String, String)> {
    let mut q: Option<String> = None;
    let mut stype = SearchType::All;
    let mut limit = DEFAULT_LIMIT;
    let mut offset = 0u32;

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let val = kv.next().unwrap_or("");
        match key {
            "q" | "query" => {
                let decoded = urlencoding::decode(val)
                    .map(|c| c.into_owned())
                    .unwrap_or_else(|_| val.to_string());
                q = Some(decoded);
            }
            "type" => {
                stype = match val {
                    "all" | "" => SearchType::All,
                    "albums" => SearchType::One(Category::Albums),
                    "tracks" => SearchType::One(Category::Tracks),
                    "artists" => SearchType::One(Category::Artists),
                    "playlists" => SearchType::One(Category::Playlists),
                    other => {
                        return Err((
                            format!("unknown type '{other}'"),
                            "type: all | albums | tracks | artists | playlists".into(),
                        ))
                    }
                };
            }
            "limit" => {
                if let Ok(n) = val.parse::<u32>() {
                    limit = n.clamp(1, MAX_LIMIT);
                }
            }
            "offset" => {
                if let Ok(n) = val.parse::<u32>() {
                    offset = n;
                }
            }
            _ => {}
        }
    }

    let q = match q {
        Some(q) if !q.trim().is_empty() => q,
        _ => {
            return Err((
                "search requires a query".into(),
                "usage: qbzd search <QUERY>".into(),
            ))
        }
    };
    Ok(SearchParams {
        q,
        stype,
        limit,
        offset,
    })
}

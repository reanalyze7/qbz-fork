pub(super) const DEFAULT_LIMIT: u32 = 20;
pub(super) const MAX_LIMIT: u32 = 100;

pub(super) fn pairs(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut kv = p.splitn(2, '=');
            let k = kv.next().unwrap_or("").to_string();
            let raw = kv.next().unwrap_or("");
            let v = urlencoding::decode(raw).map(|c| c.into_owned()).unwrap_or_else(|_| raw.to_string());
            (k, v)
        })
        .collect()
}

pub(super) fn get<'a>(p: &'a [(String, String)], key: &str) -> Option<&'a str> {
    p.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()).filter(|v| !v.is_empty())
}

pub(super) fn parse_genre(v: Option<&str>) -> Option<Vec<u64>> {
    let ids: Vec<u64> = v?.split(',').filter_map(|s| s.trim().parse::<u64>().ok()).collect();
    if ids.is_empty() {
        None
    } else {
        Some(ids)
    }
}

pub(super) fn limit_offset(p: &[(String, String)]) -> (u32, u32) {
    let limit = get(p, "limit").and_then(|v| v.parse::<u32>().ok()).map(|n| n.clamp(1, MAX_LIMIT)).unwrap_or(DEFAULT_LIMIT);
    let offset = get(p, "offset").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
    (limit, offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_genre_reads_csv_or_none() {
        assert_eq!(parse_genre(Some("1,2,3")), Some(vec![1, 2, 3]));
        assert_eq!(parse_genre(Some("64")), Some(vec![64]));
        assert_eq!(parse_genre(Some("")), None);
        assert_eq!(parse_genre(None), None);
        assert_eq!(parse_genre(Some("x,y")), None);
    }

    #[test]
    fn get_and_limit_offset_defaults() {
        let p = pairs("section=most-streamed&limit=500&genre=64");
        assert_eq!(get(&p, "section"), Some("most-streamed"));
        assert_eq!(get(&p, "missing"), None);
        assert_eq!(limit_offset(&p), (MAX_LIMIT, 0));
    }
}

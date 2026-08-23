/// Percent-decoded (key, value) pairs from a query string.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_decodes_values() {
        let p = pairs("type=track&limit=50");
        assert_eq!(p, vec![("type".into(), "track".into()), ("limit".into(), "50".into())]);
    }
}

/// The path component of a URL prefix (everything before `?`): strips a
/// `scheme://authority` head when present; a bare path passes through.
pub(super) fn url_path(prefix: &str) -> &str {
    match prefix.find("://") {
        Some(i) => {
            let rest = &prefix[i + 3..];
            match rest.find('/') {
                Some(j) => &rest[j..],
                None => "",
            }
        }
        None => prefix,
    }
}

/// Extract the authorization code from a `&`-joined query string.
/// `code_autorisation` wins over `code` (desktop parity).
pub(super) fn code_from_query(query: &str) -> Option<String> {
    let mut code_aut: Option<String> = None;
    let mut code_plain: Option<String> = None;
    for pair in query.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        match k {
            "code_autorisation" => code_aut = decode(v),
            "code" => code_plain = decode(v),
            _ => {}
        }
    }
    code_aut.or(code_plain)
}

fn decode(v: &str) -> Option<String> {
    urlencoding::decode(v).ok().map(|s| s.into_owned())
}

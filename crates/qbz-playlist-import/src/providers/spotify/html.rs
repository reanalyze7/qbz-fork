//! Tiny shared `<script id="...">` HTML extractor used by the embed scraper.

pub(super) fn extract_script(html: &str, id: &str) -> Option<String> {
    let marker = format!("id=\"{}\"", id);
    let start = html.find(&marker)?;
    let script_start = html[start..].find('>')? + start + 1;
    let script_end = html[script_start..].find("</script>")? + script_start;
    Some(html[script_start..script_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn extract_script_pulls_next_data_payload() {
        let html = concat!(
            "<html><head></head><body>",
            "<script id=\"__NEXT_DATA__\" type=\"application/json\">",
            "{\"props\":{\"pageProps\":{\"state\":{\"data\":{\"entity\":{\"title\":\"My Mix\"}}}}}}",
            "</script></body></html>"
        );
        let json_text = extract_script(html, "__NEXT_DATA__").expect("script found");
        let data: Value = serde_json::from_str(&json_text).expect("valid JSON");
        let title = data["props"]["pageProps"]["state"]["data"]["entity"]["title"]
            .as_str()
            .unwrap();
        assert_eq!(title, "My Mix");
    }

    #[test]
    fn extract_script_missing_id_is_none() {
        assert_eq!(extract_script("<html></html>", "__NEXT_DATA__"), None);
    }
}

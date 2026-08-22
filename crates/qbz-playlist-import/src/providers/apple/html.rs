//! HTML scraping helpers: `<script>`/`<meta>` extraction and entity unescaping.

pub(super) fn extract_script(html: &str, id: &str) -> Option<String> {
    let marker = format!("id=\"{}\"", id);
    let start = html.find(&marker)?;
    let script_start = html[start..].find('>')? + start + 1;
    let script_end = html[script_start..].find("</script>")? + script_start;
    let raw = &html[script_start..script_end];
    Some(unescape_basic(raw))
}

pub(super) fn extract_meta(html: &str, property: &str) -> Option<String> {
    let needle = format!("property=\"{}\"", property);
    let start = html.find(&needle)?;
    let content_start = html[start..].find("content=\"")? + start + "content=\"".len();
    let content_end = html[content_start..].find('"')? + content_start;
    Some(unescape_basic(&html[content_start..content_end]))
}

fn unescape_basic(input: &str) -> String {
    input
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_script_unescapes_serialized_server_data() {
        let html = concat!(
            "<script type=\"application/json\" id=\"serialized-server-data\">",
            "[{&quot;itemKind&quot;:&quot;trackLockup&quot;}]",
            "</script>"
        );
        assert_eq!(
            extract_script(html, "serialized-server-data").as_deref(),
            Some("[{\"itemKind\":\"trackLockup\"}]")
        );
    }

    #[test]
    fn extract_meta_reads_og_tags() {
        let html = concat!(
            "<meta property=\"og:title\" content=\"My Playlist &amp; More\">",
            "<meta property=\"og:description\" content=\"\">"
        );
        assert_eq!(
            extract_meta(html, "og:title").as_deref(),
            Some("My Playlist & More")
        );
        assert_eq!(extract_meta(html, "og:description").as_deref(), Some(""));
        assert_eq!(extract_meta(html, "og:image"), None);
    }

    #[test]
    fn unescape_basic_entities() {
        assert_eq!(
            unescape_basic("&quot;a&quot; &#34;b&#34; &amp; &lt;c&gt;"),
            "\"a\" \"b\" & <c>"
        );
    }
}

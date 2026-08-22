/// Read a short, log-safe preview of a response body — for diagnosing an
/// unexpected non-2xx (e.g. distinguishing an edge/WAF HTML 403 from the API's
/// JSON error envelope, issue #637). Bounded so a large/HTML body can't bloat
/// the log; prefixed with " : " so it reads well appended to an error message.
pub(crate) async fn body_preview(response: reqwest::Response) -> String {
    match response.text().await {
        Ok(body) => {
            let trimmed = body.trim();
            if trimmed.is_empty() {
                " : <empty body>".to_string()
            } else {
                let preview: String = trimmed.chars().take(200).collect();
                format!(" : {preview}")
            }
        }
        Err(_) => String::new(),
    }
}

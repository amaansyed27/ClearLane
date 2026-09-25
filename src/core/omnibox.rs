use url::Url;

const SEARCH_ENDPOINT: &str = "https://www.google.com/search?q=";

pub fn normalize_omnibox(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    if let Ok(url) = Url::parse(input) {
        if matches!(url.scheme(), "http" | "https") {
            return Some(url.into());
        }
    }

    if !input.chars().any(char::is_whitespace) && !input.contains('\\') {
        let lower = input.to_ascii_lowercase();
        let local = lower == "localhost"
            || lower.starts_with("localhost:")
            || lower.starts_with("127.")
            || lower.starts_with("[::1]");
        let host_like = local || input.contains('.') || input.contains(':');
        if host_like {
            let scheme = if local { "http" } else { "https" };
            let candidate = format!("{scheme}://{input}");
            if Url::parse(&candidate).is_ok() {
                return Some(candidate);
            }
        }
    }

    let encoded: String = url::form_urlencoded::byte_serialize(input.as_bytes()).collect();
    Some(format!("{SEARCH_ENDPOINT}{encoded}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_http_urls() {
        assert_eq!(
            normalize_omnibox(" https://example.com/a "),
            Some("https://example.com/a".into())
        );
    }

    #[test]
    fn adds_https_to_hosts() {
        assert_eq!(
            normalize_omnibox("example.com/path"),
            Some("https://example.com/path".into())
        );
    }

    #[test]
    fn localhost_defaults_to_http() {
        assert_eq!(
            normalize_omnibox("localhost:3000"),
            Some("http://localhost:3000".into())
        );
    }

    #[test]
    fn searches_plain_text() {
        assert_eq!(
            normalize_omnibox("clear lane browser"),
            Some("https://www.google.com/search?q=clear+lane+browser".into())
        );
    }

    #[test]
    fn does_not_open_file_or_custom_schemes() {
        let result = normalize_omnibox("file:///C:/secret.txt").unwrap();
        assert!(result.starts_with("https://www.google.com/search?q="));
        let custom = normalize_omnibox("mailto:test@example.com").unwrap();
        assert!(custom.starts_with("https://www.google.com/search?q="));
    }
}

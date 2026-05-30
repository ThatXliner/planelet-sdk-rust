use base64::{engine::general_purpose::STANDARD, Engine};

/// Decodes a base64-encoded raw body into bytes.
pub fn decode_raw_body(raw_body_base64: &str) -> Vec<u8> {
    STANDARD
        .decode(raw_body_base64)
        .unwrap_or_else(|_| Vec::new())
}

/// Decodes a base64-encoded raw body into a UTF-8 string.
pub fn decode_raw_body_text(raw_body_base64: &str) -> String {
    String::from_utf8(decode_raw_body(raw_body_base64)).unwrap_or_default()
}

/// Returns the first value for a given header name (case-insensitive).
pub fn first_header<'a>(
    headers: &'a std::collections::HashMap<String, Vec<String>>,
    name: &str,
) -> Option<&'a str> {
    let normalized = name.to_lowercase();
    for (key, values) in headers {
        if key.to_lowercase() == normalized {
            return values.first().map(|s| s.as_str());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_decode_raw_body() {
        let encoded = STANDARD.encode(b"hello world");
        let decoded = decode_raw_body(&encoded);
        assert_eq!(decoded, b"hello world");
    }

    #[test]
    fn test_decode_raw_body_text() {
        let encoded = STANDARD.encode("hello world");
        let decoded = decode_raw_body_text(&encoded);
        assert_eq!(decoded, "hello world");
    }

    #[test]
    fn test_first_header() {
        let mut headers = HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            vec!["application/json".to_string()],
        );
        headers.insert(
            "X-Custom".to_string(),
            vec!["first".to_string(), "second".to_string()],
        );

        assert_eq!(
            first_header(&headers, "content-type"),
            Some("application/json")
        );
        assert_eq!(first_header(&headers, "x-custom"), Some("first"));
        assert_eq!(first_header(&headers, "missing"), None);
    }
}

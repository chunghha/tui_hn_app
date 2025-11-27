use html2text::from_read;

use once_cell::sync::Lazy;
use regex::Regex;

static IMG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)<img\s+[^>]*alt=["']([^"']*)["'][^>]*>"#).unwrap());

/// Extracts readable text from an HTML string.
/// Strips tags and decodes basic entities using `html2text` crate.
/// Also replaces <img> tags with [Image: alt] placeholders.
pub fn extract_text_from_html(html: &str) -> String {
    // Pre-process HTML to replace images with text placeholders
    let html_with_placeholders = IMG_REGEX.replace_all(html, "[Image: $1]");

    // html2text emits wrapped lines; we can join them for now.
    let mut bytes = html_with_placeholders.as_bytes();
    from_read(&mut bytes, 80).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_text() {
        let html = "<p>Hello <strong>World</strong> &amp; friends</p>";
        let out = extract_text_from_html(html);
        assert!(out.contains("Hello"));
        assert!(out.contains("World"));
        assert!(out.contains("& friends") || out.contains("& friends"));
    }

    #[test]
    fn replaces_images_with_placeholders() {
        let html = "<p>Check this out: <img src=\"foo.jpg\" alt=\"Cool Image\" /></p>";
        let out = extract_text_from_html(html);
        assert!(out.contains("Check this out:"));
        assert!(out.contains("[Image: Cool Image]"));

        let html_single = "<img src='foo.jpg' alt='Single Quote' />";
        let out_single = extract_text_from_html(html_single);
        assert!(out_single.contains("[Image: Single Quote]"));

        let html_mixed = "<img alt=\"Mixed Attrs\" src=\"foo.jpg\" />";
        let out_mixed = extract_text_from_html(html_mixed);
        assert!(out_mixed.contains("[Image: Mixed Attrs]"));
    }
}

/// Extract domain from a URL string
/// Returns the host/domain portion without the scheme and path
/// Example: "https://github.com/foo/bar" -> Some("github.com")
pub fn extract_domain(url: &str) -> Option<String> {
    // Handle common URL formats
    let url = url.trim();

    // Remove scheme if present
    let without_scheme = if let Some(idx) = url.find("://") {
        &url[idx + 3..]
    } else {
        url
    };

    // Extract host before path or query
    let host = without_scheme
        .split('/')
        .next()?
        .split('?')
        .next()?
        .split('#')
        .next()?;

    // Remove port if present
    let domain = host.split(':').next()?;

    if domain.is_empty() {
        None
    } else {
        Some(domain.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_with_https() {
        assert_eq!(
            extract_domain("https://github.com/user/repo"),
            Some("github.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_with_http() {
        assert_eq!(
            extract_domain("http://example.com/path"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_without_scheme() {
        assert_eq!(
            extract_domain("example.com/path"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_with_port() {
        assert_eq!(
            extract_domain("https://localhost:8080/path"),
            Some("localhost".to_string())
        );
    }

    #[test]
    fn test_extract_domain_with_query() {
        assert_eq!(
            extract_domain("https://example.com?param=value"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_extract_domain_empty() {
        assert_eq!(extract_domain(""), None);
    }

    #[test]
    fn test_extract_domain_subdomain() {
        assert_eq!(
            extract_domain("https://news.ycombinator.com/item?id=123"),
            Some("news.ycombinator.com".to_string())
        );
    }
}

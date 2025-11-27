#![allow(dead_code, unused_imports)]

use tui_hn_app::utils::html::extract_text_from_html;

#[test]
fn should_extract_text_from_simple_html() {
    let html = "<html><head><title>Test</title></head><body><h1>Hello</h1><p>World &amp; others</p></body></html>";
    let text = extract_text_from_html(html);
    // Expect tags removed and entities decoded minimally
    assert!(text.contains("Hello"), "Should contain heading text");
    assert!(
        text.contains("World & others"),
        "Should decode &amp; entity"
    );
    assert!(!text.contains("<h1>"), "Tags should be stripped");
}

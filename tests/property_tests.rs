use proptest::prelude::*;
use tui_hn_app::config::AppConfig;
use tui_hn_app::internal::ui::view::calculate_wrapped_title;

proptest! {
    #[test]
    fn test_calculate_wrapped_title_no_panic(s in "\\PC*", width in 0u16..200, prefix in 0u16..50) {
        // Ensure it never panics regardless of input
        let _ = calculate_wrapped_title(&s, width, prefix);
    }

    #[test]
    fn test_calculate_wrapped_title_length(s in "[a-zA-Z0-9 ]*", width in 20u16..200) {
        let prefix = 10;
        let wrapped = calculate_wrapped_title(&s, width, prefix);

        // Check that we get output
        if !s.is_empty() {
            assert!(!wrapped.is_empty());
        }
    }

    #[test]
    fn test_config_parsing_resilience(s in "\\PC*") {
        // Fuzz the config loader with random strings
        // It should return an Err, but not panic
        let _ = ron::from_str::<AppConfig>(&s);
    }
}

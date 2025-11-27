// gpui::Hsla import removed
// hsla_to_hex removed as it depended on gpui::Hsla

//! Helpers for small theme-related textual utilities.
//!
//! This module provides a tiny helper to toggle the textual token "Dark" <-> "Light"
//! inside a theme name string. The function only affects the first standalone word
//! occurrence of "Dark" or "Light" (word boundaries). It attempts to preserve
//! simple capitalization patterns (ALL CAPS, all lower, Capitalized). If no
//! standalone token is present it appends the opposite suffix guided by a runtime
//! hint.
//!
//! Examples:
//! - "Flexoki Dark" -> "Flexoki Light"
//! - "FlexokiDark" -> "FlexokiDark Light" (no standalone token, append based on hint)
//! - "Darkness Dark" -> "Darkness Light" (replaces only the standalone token)

use regex::Regex;

/// Toggle the textual token "Dark" <-> "Light" inside a configured theme name.
///
/// - Searches for the first standalone occurrence of `Dark` or `Light` (word boundaries).
/// - Replaces only the first match and preserves a simple capitalization pattern:
///   * ALL UPPER -> ALL UPPER
///   * all lower -> all lower
///   * Capitalized -> Capitalized
/// - If no standalone token is found, appends `" Light"` or `" Dark"` depending
///   on the `runtime_is_dark` hint. If the hint is `None`, defaults to appending
///   `" Light"`.
///
/// Parameters:
/// - `name`: theme name to transform
/// - `runtime_is_dark`: optional runtime hint where `Some(true)` means runtime is dark,
///   so we append `" Light"` if no token is found (see tests).
///
/// Returns a new `String` with the modified theme name.
#[allow(dead_code)]
pub fn toggle_dark_light(name: &str, runtime_is_dark: Option<bool>) -> String {
    // Regex: case-insensitive, match whole word dark or light
    let re = Regex::new(r"(?i)\b(dark|light)\b").expect("regex compiles");
    if let Some(mat) = re.find(name) {
        let matched = mat.as_str();
        let replacement_base = if matched.eq_ignore_ascii_case("dark") {
            "Light"
        } else {
            "Dark"
        };
        let replacement = preserve_case(matched, replacement_base);
        // Build output replacing only the first occurrence
        let mut out = String::with_capacity(name.len() + 6);
        out.push_str(&name[..mat.start()]);
        out.push_str(&replacement);
        out.push_str(&name[mat.end()..]);
        out
    } else {
        // Append based on runtime hint; default to Light
        let to_append = match runtime_is_dark {
            Some(true) => " Light",
            Some(false) => " Dark",
            None => " Light",
        };
        format!("{}{}", name, to_append)
    }
}

/// Preserve simple capitalization patterns from `orig` onto `replacement`.
///
/// - If `orig` is ALL UPPER, returned replacement is uppercased.
/// - If `orig` is all lower, returned replacement is lowercased.
/// - Otherwise, capitalizes the first grapheme of `replacement` (best-effort).
#[allow(dead_code)]
fn preserve_case(orig: &str, replacement: &str) -> String {
    if orig.chars().all(|c| !c.is_alphabetic() || c.is_uppercase())
        && orig.chars().any(|c| c.is_uppercase())
        && !orig.chars().any(|c| c.is_lowercase())
    {
        // ALL UPPER (non-alpha chars allowed, but we require at least one uppercase
        replacement.to_uppercase()
    } else if orig.chars().all(|c| !c.is_alphabetic() || c.is_lowercase())
        && orig.chars().any(|c| c.is_lowercase())
        && !orig.chars().any(|c| c.is_uppercase())
    {
        // all lower
        replacement.to_lowercase()
    } else {
        // Capitalize first character of replacement (best-effort)
        let mut chars = replacement.chars();
        if let Some(first) = chars.next() {
            let first_up = first.to_uppercase().collect::<String>();
            let rest = chars.as_str();
            format!("{first_up}{rest}")
        } else {
            replacement.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Tests for the new toggle_dark_light behavior ---

    #[test]
    fn toggle_dark_to_light() {
        let got = toggle_dark_light("Flexoki Dark", Some(false));
        assert_eq!(got, "Flexoki Light");
    }

    #[test]
    fn toggle_light_to_dark() {
        let got = toggle_dark_light("Flexoki Light", Some(true));
        assert_eq!(got, "Flexoki Dark");
    }

    #[test]
    fn append_light_when_runtime_dark_and_no_token() {
        let got = toggle_dark_light("Flexoki", Some(true));
        assert_eq!(got, "Flexoki Light");
    }

    #[test]
    fn append_dark_when_runtime_light_and_no_token() {
        let got = toggle_dark_light("Flexoki", Some(false));
        assert_eq!(got, "Flexoki Dark");
    }

    #[test]
    fn replace_only_first_occurrence() {
        let got = toggle_dark_light("Darkness Dark", Some(false));
        assert_eq!(got, "Darkness Light");
    }

    #[test]
    fn preserve_all_caps() {
        let got = toggle_dark_light("SOMETHING DARK", Some(false));
        assert_eq!(got, "SOMETHING LIGHT");
    }

    #[test]
    fn preserve_lowercase() {
        let got = toggle_dark_light("something dark", Some(false));
        assert_eq!(got, "something light");
    }
}

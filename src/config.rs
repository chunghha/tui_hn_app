use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub font_sans: String,
    pub font_serif: String,
    #[allow(dead_code)]
    pub font_mono: String,
    /// Preferred theme name to apply (e.g., "Flexoki Light" / "Flexoki Dark")
    #[serde(default = "default_theme_name")]
    pub theme_name: String,
    /// Path to a specific theme file or directory to load (e.g., "./themes" or "./themes/flexoki.json")
    /// Defaults to "./themes" so ThemeRegistry can watch that directory.
    #[serde(default = "default_theme_file")]
    pub theme_file: String,
    /// If true, automatically switch a configured dark theme to its light variant
    /// when the runtime environment indicates a special terminal (e.g. TERM=xterm-ghostty)
    /// or when automatic switching is desired. Defaults to true.
    #[serde(default = "default_auto_switch_dark_to_light")]
    pub auto_switch_dark_to_light: bool,
    /// The TERM value that should be recognized as the special \"ghost\" terminal
    /// where explicit Dark/Light variants in `theme_name` must be honored instead
    /// of being auto-switched. Defaults to \"xterm-ghostty\".
    #[serde(default = "default_ghost_term_name")]
    pub ghost_term_name: String,
    /// WebView zoom level as percentage (e.g., 120 for 120%)
    #[serde(default = "default_webview_zoom")]
    pub webview_zoom: u32,
    /// How the app should inject theme colors into the WebView content.
    /// Options (config accepts lowercase strings):
    /// - "none": don't inject
    /// - "light": inject only when app theme is light
    /// - "dark": inject only when app theme is dark
    /// - "both": inject for both themes
    #[serde(default = "default_webview_theme_injection")]
    pub webview_theme_injection: String,
    /// How to apply theme injection: "invasive" (uses !important) or "css-vars" (sets CSS variables)
    #[serde(default = "default_webview_theme_mode")]
    pub webview_theme_mode: String,
    /// Maximum run length before inserting soft-wrap break characters.
    /// Set to 0 to disable the soft-wrap insertion behavior.
    #[serde(default = "default_soft_wrap_max_run")]
    pub soft_wrap_max_run: usize,
    /// Window width in pixels
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    /// Window height in pixels
    #[serde(default = "default_window_height")]
    pub window_height: f32,
}

fn default_webview_theme_injection() -> String {
    // Default to not injecting theme into WebView content.
    // Unknown/absent config -> treat as "none" (do not inject).
    "none".to_string()
}

fn default_webview_zoom() -> u32 {
    120
}

fn default_webview_theme_mode() -> String {
    // Default to invasive mode to preserve current behavior
    "invasive".to_string()
}

/// Default maximum run length before inserting soft-wrap characters.
/// A value of 0 disables the soft-wrap behavior.
fn default_soft_wrap_max_run() -> usize {
    20
}

fn default_window_width() -> f32 {
    980.0
}

fn default_window_height() -> f32 {
    720.0
}

fn default_theme_name() -> String {
    "Flexoki Light".to_string()
}

fn default_theme_file() -> String {
    "./themes".to_string()
}

fn default_auto_switch_dark_to_light() -> bool {
    true
}

fn default_ghost_term_name() -> String {
    "xterm-ghostty".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            font_sans: "IBM Plex Sans".to_string(),
            font_serif: "IBM Plex Serif".to_string(),
            font_mono: "IBM Plex Mono".to_string(),
            theme_name: default_theme_name(),
            theme_file: default_theme_file(),
            auto_switch_dark_to_light: default_auto_switch_dark_to_light(),
            ghost_term_name: default_ghost_term_name(),
            webview_zoom: 120,
            webview_theme_injection: default_webview_theme_injection(),
            webview_theme_mode: default_webview_theme_mode(),
            soft_wrap_max_run: default_soft_wrap_max_run(),
            window_width: 980.0,
            window_height: 720.0,
        }
    }
}

#[allow(dead_code)]
impl AppConfig {
    pub fn load() -> Self {
        // Look for config.ron in current directory or next to executable
        let mut candidates = Vec::new();

        // 1. Current working directory
        candidates.push(PathBuf::from("config.ron"));

        // 2. Next to executable
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            candidates.push(dir.join("config.ron"));
        }

        for path in candidates {
            if path.exists()
                && let Ok(content) = fs::read_to_string(&path)
            {
                match ron::from_str::<AppConfig>(&content) {
                    Ok(config) => {
                        tracing::info!("Loaded config from {}", path.display());
                        return config;
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse config at {}: {}", path.display(), e);
                    }
                }
            }
        }

        tracing::info!("No config file found, using defaults");
        Self::default()
    }

    pub fn save(&self) {
        self.save_to(PathBuf::from("config.ron"));
    }

    pub fn save_to(&self, path: PathBuf) {
        // Try to read existing config to preserve comments
        let existing_content = fs::read_to_string(&path).unwrap_or_default();

        if existing_content.is_empty() {
            // Fallback to standard serialization if file doesn't exist or is empty
            let pretty = ron::ser::PrettyConfig::default()
                .depth_limit(2)
                .separate_tuple_members(true)
                .enumerate_arrays(true);

            match ron::ser::to_string_pretty(self, pretty) {
                Ok(content) => {
                    if let Err(e) = fs::write(&path, content) {
                        tracing::error!("Failed to write config to {}: {}", path.display(), e);
                    } else {
                        tracing::info!("Saved config to {}", path.display());
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to serialize config: {}", e);
                }
            }
            return;
        }

        // Helper to replace value in RON content
        // Matches `key: value` or `key: "value"`
        let mut new_content = existing_content.clone();

        let replace_str = |content: &mut String, key: &str, value: &str| {
            let re = RegexBuilder::new(&format!(r#"(\s*{}\s*:\s*)"[^"]*""#, regex::escape(key)))
                .build()
                .unwrap();
            *content = re
                .replace_all(content, format!(r#"${{1}}"{}""#, value))
                .to_string();
        };

        let replace_val = |content: &mut String, key: &str, value: String| {
            let re = RegexBuilder::new(&format!(r#"(\s*{}\s*:\s*)[^,\s)]+"#, regex::escape(key)))
                .build()
                .unwrap();
            *content = re
                .replace_all(content, format!(r#"${{1}}{}"#, value))
                .to_string();
        };

        replace_str(&mut new_content, "font_sans", &self.font_sans);
        replace_str(&mut new_content, "font_serif", &self.font_serif);
        replace_str(&mut new_content, "font_mono", &self.font_mono);
        replace_str(&mut new_content, "theme_name", &self.theme_name);
        replace_str(&mut new_content, "theme_file", &self.theme_file);
        // Ensure boolean and ghost-term keys are updated when saving config so a
        // minimal config file containing these keys will be preserved/updated.
        replace_val(
            &mut new_content,
            "auto_switch_dark_to_light",
            self.auto_switch_dark_to_light.to_string(),
        );
        replace_str(&mut new_content, "ghost_term_name", &self.ghost_term_name);
        replace_val(
            &mut new_content,
            "webview_zoom",
            self.webview_zoom.to_string(),
        );
        replace_str(
            &mut new_content,
            "webview_theme_injection",
            &self.webview_theme_injection,
        );
        replace_str(
            &mut new_content,
            "webview_theme_mode",
            &self.webview_theme_mode,
        );
        replace_val(
            &mut new_content,
            "soft_wrap_max_run",
            self.soft_wrap_max_run.to_string(),
        );
        // Floating point numbers might need specific formatting, but to_string() is usually fine for RON
        replace_val(
            &mut new_content,
            "window_width",
            format!("{:.1}", self.window_width),
        );
        replace_val(
            &mut new_content,
            "window_height",
            format!("{:.1}", self.window_height),
        );

        if let Err(e) = fs::write(&path, new_content) {
            tracing::error!("Failed to update config at {}: {}", path.display(), e);
        } else {
            tracing::info!("Updated config at {} (preserving comments)", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_save_preserves_comments() {
        use std::io::Write;

        // Create a temporary config file with comments
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("config_test_comments.ron");

        let initial_content = r#"(
    // This is a comment about fonts
    font_sans: "Test Sans",
    font_serif: "Test Serif",
    font_mono: "Test Mono",

    // Theme settings
    theme_name: "Old Theme",
    theme_file: "./themes",

    // Zoom level
    webview_zoom: 100,

    // Injection mode
    webview_theme_injection: "none",

    soft_wrap_max_run: 20,
    window_width: 800.0,
    window_height: 600.0,
)"#;

        {
            let mut file = fs::File::create(&config_path).unwrap();
            file.write_all(initial_content.as_bytes()).unwrap();
        }

        // Load config manually (since load() logic is complex with paths)
        let mut config: AppConfig = ron::from_str(initial_content).unwrap();

        // Modify values
        config.webview_theme_injection = "both".to_string();
        config.webview_zoom = 150;

        // Save to the temp path
        config.save_to(config_path.clone());

        // Read back
        let new_content = fs::read_to_string(&config_path).unwrap();

        // Verify values updated
        assert!(new_content.contains("webview_theme_injection: \"both\""));
        assert!(new_content.contains("webview_zoom: 150"));

        // Verify comments preserved
        assert!(new_content.contains("// This is a comment about fonts"));
        assert!(new_content.contains("// Theme settings"));
        assert!(new_content.contains("// Zoom level"));
        assert!(new_content.contains("// Injection mode"));

        // Cleanup
        let _ = fs::remove_file(config_path);
    }
}

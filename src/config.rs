use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::internal::ui::app::Action;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct KeyBindingConfig {
    #[serde(default)]
    pub global: HashMap<String, Action>,
    #[serde(default)]
    pub list: HashMap<String, Action>,
    #[serde(default)]
    pub story_detail: HashMap<String, Action>,
    #[serde(default)]
    pub article: HashMap<String, Action>,
    #[serde(default)]
    pub bookmarks: HashMap<String, Action>,
    #[serde(default)]
    pub history: HashMap<String, Action>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct UIConfig {
    pub padding: PaddingConfig,
    pub status_bar_format: String,
    pub list_view: ListViewConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct PaddingConfig {
    pub horizontal: u16,
    pub vertical: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct ListViewConfig {
    pub show_domain: bool,
    pub show_score: bool,
    pub show_comments: bool,
    pub show_age: bool,
    pub show_author: bool,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            padding: PaddingConfig::default(),
            // Richer default format to match v0.6.3 experience
            status_bar_format:
                "{spinner} {mode} | {category} | {count}/{total} | {sort} {order} | {shortcuts} {loading_text}"
                    .to_string(),
            list_view: ListViewConfig::default(),
        }
    }
}

impl Default for PaddingConfig {
    fn default() -> Self {
        Self {
            horizontal: 1,
            vertical: 0,
        }
    }
}

impl Default for ListViewConfig {
    fn default() -> Self {
        Self {
            show_domain: true,
            show_score: true,
            show_comments: true,
            show_age: true,
            show_author: true,
        }
    }
}

/// Network retry configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct NetworkConfig {
    /// Maximum number of retry attempts (0 = no retries)
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_retry_delay_ms: u64,
    /// Maximum retry delay in milliseconds (caps exponential backoff)
    pub max_retry_delay_ms: u64,
    /// Whether to retry on timeout errors
    pub retry_on_timeout: bool,
    /// Maximum number of concurrent requests
    #[serde(default = "default_concurrent_requests")]
    pub concurrent_requests: usize,
    /// Rate limit in requests per second
    #[serde(default = "default_rate_limit_per_second")]
    pub rate_limit_per_second: f64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_retry_delay_ms: 500,
            max_retry_delay_ms: 5000,
            retry_on_timeout: true,
            concurrent_requests: default_concurrent_requests(),
            rate_limit_per_second: default_rate_limit_per_second(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct LogConfig {
    pub level: LogLevel,
    pub module_levels: HashMap<String, LogLevel>,
    pub enable_performance_metrics: bool,
    pub log_directory: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            module_levels: HashMap::new(),
            enable_performance_metrics: false,
            log_directory: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct AppConfig {
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
    /// Custom keybindings
    #[serde(default)]
    pub keybindings: Option<KeyBindingConfig>,
    /// UI customization settings
    #[serde(default)]
    pub ui: UIConfig,
    /// Network retry settings
    #[serde(default)]
    pub network: NetworkConfig,
    /// Logging configuration
    #[serde(default)]
    pub logging: LogConfig,
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

fn default_concurrent_requests() -> usize {
    10
}

fn default_rate_limit_per_second() -> f64 {
    3.0
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_name: default_theme_name(),
            theme_file: default_theme_file(),
            auto_switch_dark_to_light: default_auto_switch_dark_to_light(),
            ghost_term_name: default_ghost_term_name(),
            keybindings: None,
            ui: UIConfig::default(),
            network: NetworkConfig::default(),
            logging: LogConfig::default(),
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
                Ok(content) => match fs::write(&path, content) {
                    Ok(_) => {
                        tracing::info!("Saved config to {}", path.display());
                    }
                    Err(e) => {
                        tracing::error!("Failed to write config to {}: {}", path.display(), e);
                    }
                },
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

        match fs::write(&path, new_content) {
            Ok(_) => {
                tracing::info!("Updated config at {} (preserving comments)", path.display());
            }
            Err(e) => {
                tracing::error!("Failed to update config at {}: {}", path.display(), e);
            }
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
    // Theme settings
    theme_name: "Old Theme",
    theme_file: "./themes",
)"#;

        {
            let mut file = fs::File::create(&config_path).unwrap();
            file.write_all(initial_content.as_bytes()).unwrap();
        }

        // Load config manually (since load() logic is complex with paths)
        let mut config: AppConfig = ron::from_str(initial_content).unwrap();

        // Modify values
        config.theme_name = "New Theme".to_string();

        // Save to the temp path
        config.save_to(config_path.clone());

        // Read back
        let new_content = fs::read_to_string(&config_path).unwrap();

        // Verify values updated
        assert!(new_content.contains("theme_name: \"New Theme\""));

        // Verify comments preserved
        assert!(new_content.contains("// Theme settings"));

        // Cleanup
        let _ = fs::remove_file(config_path);
    }
}

use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeFile {
    #[allow(dead_code)]
    pub name: String,
    pub themes: Vec<ThemeVariant>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeVariant {
    #[allow(dead_code)]
    pub name: String,
    pub mode: String, // "light" or "dark"
    pub colors: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TuiTheme {
    pub background: Color,
    pub foreground: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub border: Color,
    #[allow(dead_code)]
    pub link: Color,
    pub score: Color,
    pub comment_author: Color,
    pub comment_time: Color,
}

impl Default for TuiTheme {
    fn default() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::Reset,
            selection_bg: Color::Blue,
            selection_fg: Color::White,
            border: Color::White,
            link: Color::Blue,
            score: Color::Yellow,
            comment_author: Color::Blue,
            comment_time: Color::DarkGray,
        }
    }
}

#[tracing::instrument(skip(path, mode), fields(path = ?path, mode = %mode))]
pub fn load_theme(path: &Path, mode: &str, enable_performance_metrics: bool) -> Result<TuiTheme> {
    let start = std::time::Instant::now();
    let content = fs::read_to_string(path).context("Failed to read theme file")?;
    let theme_file: ThemeFile =
        serde_json::from_str(&content).context("Failed to parse theme JSON")?;

    let variant = theme_file
        .themes
        .iter()
        .find(|t| t.mode == mode)
        .or_else(|| theme_file.themes.first())
        .context("No matching theme variant found")?;

    let theme = TuiTheme {
        background: parse_color(
            variant
                .colors
                .get("background")
                .unwrap_or(&"#000000".to_string()),
        ),
        foreground: parse_color(
            variant
                .colors
                .get("foreground")
                .unwrap_or(&"#ffffff".to_string()),
        ),
        selection_bg: parse_color(
            variant
                .colors
                .get("selection.background")
                .or_else(|| variant.colors.get("list.active.background"))
                .or_else(|| variant.colors.get("primary.background"))
                .unwrap_or(&"#0000ff".to_string()),
        ),
        selection_fg: parse_color(
            variant
                .colors
                .get("accent.foreground")
                .or_else(|| variant.colors.get("foreground"))
                .unwrap_or(&"#ffffff".to_string()),
        ),
        border: parse_color(
            variant
                .colors
                .get("border")
                .unwrap_or(&"#ffffff".to_string()),
        ),
        link: parse_color(
            variant
                .colors
                .get("base.blue")
                .unwrap_or(&"#0000ff".to_string()),
        ),
        score: parse_color(
            variant
                .colors
                .get("base.yellow")
                .unwrap_or(&"#ffff00".to_string()),
        ),
        comment_author: parse_color(
            variant
                .colors
                .get("base.blue")
                .unwrap_or(&"#0000ff".to_string()),
        ),
        comment_time: parse_color(
            variant
                .colors
                .get("muted.foreground")
                .unwrap_or(&"#808080".to_string()),
        ),
    };

    if enable_performance_metrics {
        tracing::debug!(elapsed = ?start.elapsed(), "Loaded theme");
    }

    Ok(theme)
}

fn parse_color(hex: &str) -> Color {
    if let Ok(c) = hex.parse::<Color>() {
        return c;
    }

    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 | 8 => {
            // For 8-char hex (with alpha), ignore the alpha and use the RGB components.
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Color::Rgb(r, g, b)
        }
        _ => Color::Reset,
    }
}

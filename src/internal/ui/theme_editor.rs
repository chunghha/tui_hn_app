use crate::utils::theme_loader::TuiTheme;
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeProperty {
    Background,
    Foreground,
    SelectionBg,
    SelectionFg,
    Border,
    Link,
    Score,
    CommentAuthor,
    CommentTime,
}

impl ThemeProperty {
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            ThemeProperty::Background => "Background",
            ThemeProperty::Foreground => "Foreground",
            ThemeProperty::SelectionBg => "Selection Background",
            ThemeProperty::SelectionFg => "Selection Foreground",
            ThemeProperty::Border => "Border",
            ThemeProperty::Link => "Link",
            ThemeProperty::Score => "Score",
            ThemeProperty::CommentAuthor => "Comment Author",
            ThemeProperty::CommentTime => "Comment Time",
        }
    }

    pub fn all() -> Vec<ThemeProperty> {
        vec![
            ThemeProperty::Background,
            ThemeProperty::Foreground,
            ThemeProperty::SelectionBg,
            ThemeProperty::SelectionFg,
            ThemeProperty::Border,
            ThemeProperty::Link,
            ThemeProperty::Score,
            ThemeProperty::CommentAuthor,
            ThemeProperty::CommentTime,
        ]
    }

    #[allow(dead_code)]
    pub fn get_color(&self, theme: &TuiTheme) -> Color {
        match self {
            ThemeProperty::Background => theme.background,
            ThemeProperty::Foreground => theme.foreground,
            ThemeProperty::SelectionBg => theme.selection_bg,
            ThemeProperty::SelectionFg => theme.selection_fg,
            ThemeProperty::Border => theme.border,
            ThemeProperty::Link => theme.link,
            ThemeProperty::Score => theme.score,
            ThemeProperty::CommentAuthor => theme.comment_author,
            ThemeProperty::CommentTime => theme.comment_time,
        }
    }

    #[allow(dead_code)]
    pub fn set_color(&self, theme: &mut TuiTheme, color: Color) {
        match self {
            ThemeProperty::Background => theme.background = color,
            ThemeProperty::Foreground => theme.foreground = color,
            ThemeProperty::SelectionBg => theme.selection_bg = color,
            ThemeProperty::SelectionFg => theme.selection_fg = color,
            ThemeProperty::Border => theme.border = color,
            ThemeProperty::Link => theme.link = color,
            ThemeProperty::Score => theme.score = color,
            ThemeProperty::CommentAuthor => theme.comment_author = color,
            ThemeProperty::CommentTime => theme.comment_time = color,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorState {
    Editing,
    Naming,
}

#[derive(Debug, Clone)]
pub struct ThemeEditor {
    pub active: bool,
    pub state: EditorState,
    pub selected_property: usize,
    #[allow(dead_code)]
    pub selected_channel: ColorChannel,
    pub editing: bool,
    pub temp_theme: TuiTheme,
    pub name_input: String,
}

impl ThemeEditor {
    pub fn new(current_theme: TuiTheme) -> Self {
        Self {
            active: false,
            state: EditorState::Editing,
            selected_property: 0,
            selected_channel: ColorChannel::Red,
            editing: false,
            temp_theme: current_theme,
            name_input: String::new(),
        }
    }

    pub fn toggle(&mut self, current_theme: &TuiTheme) {
        self.active = !self.active;
        if self.active {
            // Reset temp theme to current when opening
            self.temp_theme = current_theme.clone();
            self.selected_property = 0;
            self.editing = false;
            self.state = EditorState::Editing;
            self.name_input.clear();
        }
    }

    #[allow(dead_code)]
    pub fn navigate_property(&mut self, delta: i32) {
        let properties = ThemeProperty::all();
        let new_index =
            (self.selected_property as i32 + delta).rem_euclid(properties.len() as i32) as usize;
        self.selected_property = new_index;
    }

    #[allow(dead_code)]
    pub fn navigate_channel(&mut self, next: bool) {
        self.selected_channel = match (self.selected_channel, next) {
            (ColorChannel::Red, true) => ColorChannel::Green,
            (ColorChannel::Green, true) => ColorChannel::Blue,
            (ColorChannel::Blue, true) => ColorChannel::Red,
            (ColorChannel::Red, false) => ColorChannel::Blue,
            (ColorChannel::Green, false) => ColorChannel::Red,
            (ColorChannel::Blue, false) => ColorChannel::Green,
        };
    }

    #[allow(dead_code)]
    pub fn adjust_color(&mut self, increase: bool) {
        let properties = ThemeProperty::all();
        if self.selected_property >= properties.len() {
            return;
        }

        let property = properties[self.selected_property];
        let current_color = property.get_color(&self.temp_theme);

        match current_color {
            Color::Rgb(r, g, b) => {
                let (new_r, new_g, new_b) = match self.selected_channel {
                    ColorChannel::Red => {
                        let new_r = if increase {
                            r.saturating_add(5)
                        } else {
                            r.saturating_sub(5)
                        };
                        (new_r, g, b)
                    }
                    ColorChannel::Green => {
                        let new_g = if increase {
                            g.saturating_add(5)
                        } else {
                            g.saturating_sub(5)
                        };
                        (r, new_g, b)
                    }
                    ColorChannel::Blue => {
                        let new_b = if increase {
                            b.saturating_add(5)
                        } else {
                            b.saturating_sub(5)
                        };
                        (r, g, new_b)
                    }
                };
                property.set_color(&mut self.temp_theme, Color::Rgb(new_r, new_g, new_b));
            }
            _ => {
                // Convert non-RGB colors to RGB(128, 128, 128) as starting point
                property.set_color(&mut self.temp_theme, Color::Rgb(128, 128, 128));
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_current_property(&self) -> Option<ThemeProperty> {
        let properties = ThemeProperty::all();
        properties.get(self.selected_property).copied()
    }

    // Calculate luminance to determine if theme is dark
    // Formula: 0.2126*R + 0.7152*G + 0.0722*B
    pub fn is_dark_theme(&self) -> bool {
        match self.temp_theme.background {
            Color::Rgb(r, g, b) => {
                let lum = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
                lum < 128.0
            }
            _ => true, // Default to dark if unknown
        }
    }

    // Generate a complementary theme
    pub fn generate_complementary(&self) -> TuiTheme {
        let mut new_theme = self.temp_theme.clone();

        // Helper to invert color
        let invert = |c: Color| -> Color {
            match c {
                Color::Rgb(r, g, b) => Color::Rgb(255 - r, 255 - g, 255 - b),
                other => other,
            }
        };

        // Invert main background and foreground
        new_theme.background = invert(self.temp_theme.background);
        new_theme.foreground = invert(self.temp_theme.foreground);

        // Adjust selection
        new_theme.selection_bg = invert(self.temp_theme.selection_bg);
        new_theme.selection_fg = invert(self.temp_theme.selection_fg);

        // Keep accent colors but maybe adjust brightness if needed?
        // For now, let's just invert them too to ensure contrast,
        // or keep them if they are mid-tone.
        // A simple inversion is a good starting point for "opposite".
        new_theme.border = invert(self.temp_theme.border);
        new_theme.link = invert(self.temp_theme.link);
        new_theme.score = invert(self.temp_theme.score);
        new_theme.comment_author = invert(self.temp_theme.comment_author);
        new_theme.comment_time = invert(self.temp_theme.comment_time);

        new_theme
    }
}

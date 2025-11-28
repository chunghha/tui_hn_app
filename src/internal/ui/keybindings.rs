use crate::internal::ui::app::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Represents a view mode for context-specific keybindings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyBindingContext {
    Global,
    List,
    StoryDetail,
    Article,
    Bookmarks,
    History,
}

/// Maps key events to actions
#[derive(Debug, Clone)]
pub struct KeyBindingMap {
    global: HashMap<KeyEvent, Action>,
    list: HashMap<KeyEvent, Action>,
    story_detail: HashMap<KeyEvent, Action>,
    article: HashMap<KeyEvent, Action>,
    bookmarks: HashMap<KeyEvent, Action>,
    history: HashMap<KeyEvent, Action>,
}

impl KeyBindingMap {
    /// Create a new empty keybinding map
    pub fn new() -> Self {
        Self {
            global: HashMap::new(),
            list: HashMap::new(),
            story_detail: HashMap::new(),
            article: HashMap::new(),
            bookmarks: HashMap::new(),
            history: HashMap::new(),
        }
    }

    /// Get the action for a given key event in a specific context
    /// Checks context-specific bindings first, then falls back to global
    pub fn get_action(&self, key: &KeyEvent, context: KeyBindingContext) -> Option<Action> {
        // First check context-specific bindings
        let context_map = match context {
            KeyBindingContext::Global => &self.global,
            KeyBindingContext::List => &self.list,
            KeyBindingContext::StoryDetail => &self.story_detail,
            KeyBindingContext::Article => &self.article,
            KeyBindingContext::Bookmarks => &self.bookmarks,
            KeyBindingContext::History => &self.history,
        };

        if let Some(action) = context_map.get(key) {
            return Some(action.clone());
        }

        // Fall back to global bindings
        self.global.get(key).cloned()
    }

    /// Add a keybinding for a specific context
    pub fn add_binding(&mut self, context: KeyBindingContext, key: KeyEvent, action: Action) {
        let map = match context {
            KeyBindingContext::Global => &mut self.global,
            KeyBindingContext::List => &mut self.list,
            KeyBindingContext::StoryDetail => &mut self.story_detail,
            KeyBindingContext::Article => &mut self.article,
            KeyBindingContext::Bookmarks => &mut self.bookmarks,
            KeyBindingContext::History => &mut self.history,
        };
        map.insert(key, action);
    }

    /// Merge custom keybindings from configuration
    pub fn merge_config(&mut self, config: &crate::config::KeyBindingConfig) {
        let mut merge = |ctx: KeyBindingContext, bindings: &HashMap<String, Action>| {
            for (key_str, action) in bindings {
                if let Some(key_event) = parse_key_str(key_str) {
                    self.add_binding(ctx, key_event, action.clone());
                } else {
                    tracing::warn!("Invalid key string in config: {}", key_str);
                }
            }
        };

        merge(KeyBindingContext::Global, &config.global);
        merge(KeyBindingContext::List, &config.list);
        merge(KeyBindingContext::StoryDetail, &config.story_detail);
        merge(KeyBindingContext::Article, &config.article);
        merge(KeyBindingContext::Bookmarks, &config.bookmarks);
        merge(KeyBindingContext::History, &config.history);
    }

    /// Detect conflicts within a single context
    /// Returns a list of keys that are bound to multiple actions
    #[allow(dead_code)]
    pub fn detect_conflicts(&self, context: KeyBindingContext) -> Vec<KeyEvent> {
        let _map = match context {
            KeyBindingContext::Global => &self.global,
            KeyBindingContext::List => &self.list,
            KeyBindingContext::StoryDetail => &self.story_detail,
            KeyBindingContext::Article => &self.article,
            KeyBindingContext::Bookmarks => &self.bookmarks,
            KeyBindingContext::History => &self.history,
        };

        // In our implementation, HashMap prevents conflicts by design
        // Each key can only map to one action
        // This function is here for future extensibility
        Vec::new()
    }
}

impl Default for KeyBindingMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a key string into a KeyEvent
/// Supported formats:
/// - Single char: "j", "k", "1"
/// - Special keys: "Enter", "Tab", "Esc", "Up", "Down", "Left", "Right"
/// - With modifiers: "Ctrl+C", "Shift+Tab"
pub fn parse_key_str(key_str: &str) -> Option<KeyEvent> {
    let parts: Vec<&str> = key_str.split('+').collect();

    let mut modifiers = KeyModifiers::empty();
    let key_part = if parts.len() > 1 {
        // Has modifiers
        for modifier in &parts[..parts.len() - 1] {
            match modifier.to_lowercase().as_str() {
                "ctrl" => modifiers |= KeyModifiers::CONTROL,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "alt" => modifiers |= KeyModifiers::ALT,
                _ => return None, // Invalid modifier
            }
        }
        parts[parts.len() - 1]
    } else {
        parts[0]
    };

    let code = match key_part {
        "Enter" => KeyCode::Enter,
        "Tab" => KeyCode::Tab,
        "Esc" => KeyCode::Esc,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            KeyCode::Char(c)
        }
        _ => return None, // Unknown key
    };

    Some(KeyEvent::new(code, modifiers))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_char() {
        let key = parse_key_str("j").unwrap();
        assert_eq!(key.code, KeyCode::Char('j'));
        assert_eq!(key.modifiers, KeyModifiers::empty());
    }

    #[test]
    fn test_parse_special_key() {
        let key = parse_key_str("Enter").unwrap();
        assert_eq!(key.code, KeyCode::Enter);
    }

    #[test]
    fn test_parse_with_modifier() {
        let key = parse_key_str("Ctrl+C").unwrap();
        assert_eq!(key.code, KeyCode::Char('C'));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_keybinding_map_global_fallback() {
        let mut map = KeyBindingMap::new();
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());

        map.add_binding(KeyBindingContext::Global, key, Action::Quit);

        // Should work in any context via fallback
        assert!(matches!(
            map.get_action(&key, KeyBindingContext::List),
            Some(Action::Quit)
        ));
    }

    #[test]
    fn test_keybinding_map_context_override() {
        let mut map = KeyBindingMap::new();
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty());

        map.add_binding(KeyBindingContext::Global, key, Action::NavigateDown);
        map.add_binding(KeyBindingContext::Article, key, Action::ScrollArticleDown);

        // In article context, should use context-specific binding
        assert!(matches!(
            map.get_action(&key, KeyBindingContext::Article),
            Some(Action::ScrollArticleDown)
        ));

        // In other contexts, should use global binding
        assert!(matches!(
            map.get_action(&key, KeyBindingContext::List),
            Some(Action::NavigateDown)
        ));
    }
}

use crate::api::StoryListType;
use crate::internal::ui::app::Action;
use crate::internal::ui::keybindings::{KeyBindingContext, KeyBindingMap};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Create default keybindings for the application
pub fn create_default_keybindings() -> KeyBindingMap {
    let mut map = KeyBindingMap::new();

    // Global bindings (work in all contexts)
    add_global_bindings(&mut map);

    // View-specific bindings
    add_list_bindings(&mut map);
    add_story_detail_bindings(&mut map);
    add_article_bindings(&mut map);
    add_bookmarks_bindings(&mut map);
    add_history_bindings(&mut map);

    map
}

fn add_global_bindings(map: &mut KeyBindingMap) {
    let ctx = KeyBindingContext::Global;

    // Help
    map.add_binding(ctx, key('?'), Action::ToggleHelp);

    // Quit / Back
    map.add_binding(ctx, key('q'), Action::Back);
    map.add_binding(ctx, key_code(KeyCode::Esc), Action::Back);

    // Navigation
    map.add_binding(ctx, key('j'), Action::NavigateDown);
    map.add_binding(ctx, key('k'), Action::NavigateUp);
    map.add_binding(ctx, key_code(KeyCode::Down), Action::NavigateDown);
    map.add_binding(ctx, key_code(KeyCode::Up), Action::NavigateUp);

    // Selection
    map.add_binding(ctx, key_code(KeyCode::Enter), Action::Enter);

    // Browser
    map.add_binding(ctx, key('o'), Action::OpenBrowser);

    // Story categories
    map.add_binding(ctx, key('1'), Action::LoadStories(StoryListType::Top));
    map.add_binding(ctx, key('2'), Action::LoadStories(StoryListType::New));
    map.add_binding(ctx, key('3'), Action::LoadStories(StoryListType::Best));
    map.add_binding(ctx, key('4'), Action::LoadStories(StoryListType::Ask));
    map.add_binding(ctx, key('5'), Action::LoadStories(StoryListType::Show));
    map.add_binding(ctx, key('6'), Action::LoadStories(StoryListType::Job));

    // Sorting
    map.add_binding(ctx, key('S'), Action::SortByScore);
    map.add_binding(ctx, key('C'), Action::SortByComments);
    map.add_binding(ctx, key('T'), Action::SortByTime);
    map.add_binding(ctx, key('O'), Action::ToggleSortOrder);

    // Theme
    map.add_binding(ctx, key('t'), Action::SwitchTheme);

    // Bookmarks
    map.add_binding(ctx, key('b'), Action::ToggleBookmark);
    map.add_binding(ctx, key('B'), Action::ViewBookmarks);

    // History
    map.add_binding(ctx, key('H'), Action::ViewHistory);
}

fn add_list_bindings(map: &mut KeyBindingMap) {
    let ctx = KeyBindingContext::List;

    // Quit
    map.add_binding(ctx, key('q'), Action::Quit);
    map.add_binding(ctx, key_code(KeyCode::Esc), Action::Quit);

    // Load more stories
    map.add_binding(ctx, key('m'), Action::LoadMoreStories);
    map.add_binding(ctx, key('A'), Action::LoadAllStories);

    // Toggle search mode is handled differently as it changes InputMode
    // Not included here as it's a special case in handle_input
}

fn add_story_detail_bindings(map: &mut KeyBindingMap) {
    let ctx = KeyBindingContext::StoryDetail;

    // Toggle between Comments and Article view
    map.add_binding(ctx, key_code(KeyCode::Tab), Action::ToggleArticleView);

    // Load more comments
    map.add_binding(ctx, key('n'), Action::LoadMoreComments);
}

fn add_article_bindings(map: &mut KeyBindingMap) {
    let ctx = KeyBindingContext::Article;

    // Override navigation keys for article scrolling
    map.add_binding(ctx, key('j'), Action::ScrollArticleDown);
    map.add_binding(ctx, key('k'), Action::ScrollArticleUp);
    map.add_binding(ctx, key_code(KeyCode::Down), Action::ScrollArticleDown);
    map.add_binding(ctx, key_code(KeyCode::Up), Action::ScrollArticleUp);

    // Tab to toggle back to comments
    map.add_binding(ctx, key_code(KeyCode::Tab), Action::ToggleArticleView);
}

fn add_bookmarks_bindings(_map: &mut KeyBindingMap) {
    let _ctx = KeyBindingContext::Bookmarks;

    // Bookmarks view uses mostly global bindings
    // No specific overrides needed currently
}

fn add_history_bindings(map: &mut KeyBindingMap) {
    let ctx = KeyBindingContext::History;

    // Clear history
    map.add_binding(ctx, key('X'), Action::ClearHistory);
}

/// Helper to create a simple char key event
fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty())
}

/// Helper to create a key event from KeyCode
fn key_code(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keybindings_global() {
        let map = create_default_keybindings();
        let quit_key = key('q');

        // In List, should be Quit
        assert!(matches!(
            map.get_action(&quit_key, KeyBindingContext::List),
            Some(Action::Quit)
        ));

        // In other context (e.g. Article), should be Back
        assert!(matches!(
            map.get_action(&quit_key, KeyBindingContext::Article),
            Some(Action::Back)
        ));
    }

    #[test]
    fn test_default_keybindings_article_override() {
        let map = create_default_keybindings();
        let j_key = key('j');

        // In List, should navigate down
        assert!(matches!(
            map.get_action(&j_key, KeyBindingContext::List),
            Some(Action::NavigateDown)
        ));

        // In Article, should scroll article down
        assert!(matches!(
            map.get_action(&j_key, KeyBindingContext::Article),
            Some(Action::ScrollArticleDown)
        ));
    }
}

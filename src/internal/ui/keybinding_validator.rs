use crate::config::KeyBindingConfig;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConflictReport {
    pub description: String,
    pub keys: String,
    pub context: String,
}

pub fn detect_conflicts(config: &KeyBindingConfig) -> Vec<ConflictReport> {
    let mut conflicts = Vec::new();

    // 1. Check global keybindings
    // Since it's a HashMap<String, Action>, keys are unique by definition.
    // We just collect them to check for shadowing.
    let global_keys: HashMap<&String, &crate::internal::ui::app::Action> =
        config.global.iter().collect();

    // Helper to check a specific context map for shadowing
    let mut check_context =
        |context_name: &str, bindings: &HashMap<String, crate::internal::ui::app::Action>| {
            for (key, action) in bindings {
                // Check for shadowing of global keys
                if let Some(global_action) = global_keys.get(key) {
                    // Shadowing is allowed but we might want to warn if it's confusing
                    // For now, we'll report it as a "conflict" if the actions are different
                    // If they are the same action, it's redundant but harmless
                    if *global_action != action {
                        conflicts.push(ConflictReport {
                            description: format!(
                                "{} key '{}' shadows Global key (Global: {:?}, {}: {:?})",
                                context_name, key, global_action, context_name, action
                            ),
                            keys: key.clone(),
                            context: context_name.to_string(),
                        });
                    }
                }
            }
        };

    check_context("Story List", &config.list);
    check_context("Story Detail", &config.story_detail);
    check_context("Article View", &config.article);
    check_context("Bookmarks View", &config.bookmarks);
    check_context("History View", &config.history);

    conflicts
}

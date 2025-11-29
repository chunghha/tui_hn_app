use anyhow::{Context, Result};
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::info;

use super::models::Story;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkedStory {
    pub id: u32,
    pub title: String,
    pub url: Option<String>,
    pub bookmarked_at: Zoned,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bookmarks {
    pub stories: Vec<BookmarkedStory>,
    #[serde(skip)]
    file_path: Option<PathBuf>,
}

impl Bookmarks {
    pub fn new() -> Self {
        Self {
            stories: Vec::new(),
            file_path: None,
        }
    }

    pub fn load_or_create() -> Result<Self> {
        // Resolve the OS-specific config directory and append our app folder.
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("tui-hn-app");

        // Log the resolved config directory for diagnostics.
        info!(config_dir = %config_dir.display(), "Resolved config directory for bookmarks");

        match config_dir.exists() {
            false => {
                // Create the directory and log the creation.
                fs::create_dir_all(&config_dir).with_context(|| {
                    format!("Failed to create config directory {}", config_dir.display())
                })?;
                info!(config_dir = %config_dir.display(), "Created config directory for bookmarks");
            }
            true => {
                info!(config_dir = %config_dir.display(), "Config directory already exists");
            }
        }

        let file_path = config_dir.join("bookmarks.json");
        info!(bookmarks_file = %file_path.display(), "Resolved bookmarks file path");

        match file_path.exists() {
            true => {
                info!(bookmarks_file = %file_path.display(), "Bookmarks file exists, attempting to read");
                let content =
                    fs::read_to_string(&file_path).context("Failed to read bookmarks file")?;
                let mut bookmarks: Bookmarks =
                    serde_json::from_str(&content).context("Failed to parse bookmarks file")?;
                bookmarks.file_path = Some(file_path.clone());
                info!(bookmarks_file = %file_path.display(), "Loaded bookmarks from file");
                Ok(bookmarks)
            }
            false => {
                info!(bookmarks_file = %file_path.display(), "No bookmarks file found, initializing empty bookmarks with file path set");
                Ok(Self {
                    stories: Vec::new(),
                    file_path: Some(file_path),
                })
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        match &self.file_path {
            Some(path) => {
                let content =
                    serde_json::to_string_pretty(self).context("Failed to serialize bookmarks")?;
                fs::write(path, content).context("Failed to write bookmarks file")?;
                info!(bookmarks_file = %path.display(), "Saved bookmarks to file");
            }
            None => {
                info!("Bookmarks.save() called but no file_path is set; skipping write");
            }
        }
        Ok(())
    }

    pub fn add(&mut self, story: &Story) {
        if !self.contains(story.id) {
            let bookmarked = BookmarkedStory {
                id: story.id,
                title: story.title.clone().unwrap_or_default(),
                url: story.url.clone(),
                bookmarked_at: Zoned::now(),
            };
            // Add to beginning of list (newest first)
            self.stories.insert(0, bookmarked);
        }
    }

    pub fn remove(&mut self, id: u32) {
        self.stories.retain(|s| s.id != id);
    }

    pub fn contains(&self, id: u32) -> bool {
        self.stories.iter().any(|s| s.id == id)
    }

    pub fn toggle(&mut self, story: &Story) {
        match self.contains(story.id) {
            true => self.remove(story.id),
            false => self.add(story),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remove_bookmark() {
        let mut bookmarks = Bookmarks::new();
        let story = Story {
            id: 1,
            title: Some("Test Story".to_string()),
            url: Some("https://example.com".to_string()),
            by: Some("author".to_string()),
            score: Some(100),
            time: Some(1234567890),
            descendants: Some(10),
            kids: None,
        };

        bookmarks.add(&story);
        assert!(bookmarks.contains(1));
        assert_eq!(bookmarks.stories.len(), 1);
        assert_eq!(bookmarks.stories[0].title, "Test Story");

        bookmarks.remove(1);
        assert!(!bookmarks.contains(1));
        assert!(bookmarks.stories.is_empty());
    }

    #[test]
    fn test_toggle_bookmark() {
        let mut bookmarks = Bookmarks::new();
        let story = Story {
            id: 2,
            title: Some("Toggle Story".to_string()),
            url: None,
            by: None,
            score: None,
            time: None,
            descendants: None,
            kids: None,
        };

        bookmarks.toggle(&story);
        assert!(bookmarks.contains(2));

        bookmarks.toggle(&story);
        assert!(!bookmarks.contains(2));
    }
}

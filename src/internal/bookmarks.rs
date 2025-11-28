use anyhow::{Context, Result};
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("tui-hn-app");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?
        }

        let file_path = config_dir.join("bookmarks.json");

        match file_path.exists() {
            true => {
                let content =
                    fs::read_to_string(&file_path).context("Failed to read bookmarks file")?;
                let mut bookmarks: Bookmarks =
                    serde_json::from_str(&content).context("Failed to parse bookmarks file")?;
                bookmarks.file_path = Some(file_path);
                Ok(bookmarks)
            }
            false => Ok(Self {
                stories: Vec::new(),
                file_path: Some(file_path),
            }),
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = &self.file_path {
            let content =
                serde_json::to_string_pretty(self).context("Failed to serialize bookmarks")?;
            fs::write(path, content).context("Failed to write bookmarks file")?;
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

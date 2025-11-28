use anyhow::{Context, Result};
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use super::models::Story;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewedStory {
    pub id: u32,
    pub title: String,
    pub url: Option<String>,
    pub by: Option<String>,
    pub score: Option<u32>,
    pub descendants: Option<u32>,
    pub viewed_at: Zoned,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct History {
    pub stories: Vec<ViewedStory>,
    #[serde(skip)]
    file_path: Option<PathBuf>,
    #[serde(skip)]
    max_size: usize,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        Self {
            stories: Vec::new(),
            file_path: None,
            max_size,
        }
    }

    pub fn load_or_create(max_size: usize) -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("tui-hn-app");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        let file_path = config_dir.join("history.json");

        match file_path.exists() {
            true => {
                let content =
                    fs::read_to_string(&file_path).context("Failed to read history file")?;
                let mut history: History =
                    serde_json::from_str(&content).context("Failed to parse history file")?;
                history.file_path = Some(file_path);
                history.max_size = max_size;
                Ok(history)
            }
            false => Ok(Self {
                stories: Vec::new(),
                file_path: Some(file_path),
                max_size,
            }),
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = &self.file_path {
            let content =
                serde_json::to_string_pretty(self).context("Failed to serialize history")?;
            fs::write(path, content).context("Failed to write history file")?;
        }
        Ok(())
    }

    pub fn add(&mut self, story: &Story) {
        // Remove existing entry if present (to move it to top)
        self.stories.retain(|s| s.id != story.id);

        let viewed = ViewedStory {
            id: story.id,
            title: story.title.clone().unwrap_or_default(),
            url: story.url.clone(),
            by: story.by.clone(),
            score: story.score,
            descendants: story.descendants,
            viewed_at: Zoned::now(),
        };

        self.stories.insert(0, viewed);

        // Enforce max size
        if self.stories.len() > self.max_size {
            self.stories.truncate(self.max_size);
        }
    }

    pub fn clear(&mut self) {
        self.stories.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_history() {
        let mut history = History::new(5);
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

        history.add(&story);
        assert_eq!(history.stories.len(), 1);
        assert_eq!(history.stories[0].id, 1);
        assert_eq!(history.stories[0].title, "Test Story");

        // Add same story again, should move to top (no duplicate)
        history.add(&story);
        assert_eq!(history.stories.len(), 1);

        // Add more stories to test max size
        for i in 2..7 {
            let s = Story {
                id: i,
                title: Some(format!("Story {}", i)),
                url: None,
                by: None,
                score: None,
                time: None,
                descendants: None,
                kids: None,
            };
            history.add(&s);
        }

        // Should be capped at 5
        assert_eq!(history.stories.len(), 5);
        // Most recent (id 6) should be at index 0
        assert_eq!(history.stories[0].id, 6);
        // Oldest (id 2) should be at index 4. ID 1 should have been evicted.
        assert_eq!(history.stories[4].id, 2);
    }

    #[test]
    fn test_clear_history() {
        let mut history = History::new(5);
        let story = Story {
            id: 1,
            title: Some("Test".to_string()),
            ..Default::default()
        };
        history.add(&story);
        assert_eq!(history.stories.len(), 1);

        history.clear();
        assert!(history.stories.is_empty());
    }
}

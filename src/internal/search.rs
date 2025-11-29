use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Search mode - what to search in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    Title,
    Comments,
    TitleAndComments,
}

impl SearchMode {
    pub fn next(&self) -> Self {
        match self {
            Self::Title => Self::Comments,
            Self::Comments => Self::TitleAndComments,
            Self::TitleAndComments => Self::Title,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Comments => "Comments",
            Self::TitleAndComments => "Title+Comments",
        }
    }
}

/// Search type - how to interpret the query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchType {
    Literal,
    Regex,
}

impl SearchType {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Literal => Self::Regex,
            Self::Regex => Self::Literal,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Literal => "Literal",
            Self::Regex => "Regex",
        }
    }
}

/// Current search query with its configuration
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub mode: SearchMode,
    pub search_type: SearchType,
    #[allow(dead_code)]
    compiled_regex: Option<Regex>,
    pub regex_error: Option<String>,
}

impl SearchQuery {
    pub fn new(query: String, mode: SearchMode, search_type: SearchType) -> Self {
        let (compiled_regex, regex_error) = match search_type {
            SearchType::Regex => match Regex::new(&query) {
                Ok(re) => (Some(re), None),
                Err(e) => (None, Some(format!("Regex error: {}", e))),
            },
            SearchType::Literal => (None, None),
        };

        Self {
            query,
            mode,
            search_type,
            compiled_regex,
            regex_error,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    pub fn matches(&self, text: &str) -> bool {
        match self.search_type {
            SearchType::Literal => text.to_lowercase().contains(&self.query.to_lowercase()),
            SearchType::Regex => match &self.compiled_regex {
                Some(re) => re.is_match(text),
                None => false,
            },
        }
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            mode: SearchMode::Title,
            search_type: SearchType::Literal,
            compiled_regex: None,
            regex_error: None,
        }
    }
}

/// Search history - tracks recent searches
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchHistory {
    pub queries: Vec<String>,
    #[serde(skip)]
    file_path: Option<PathBuf>,
    #[serde(skip)]
    max_size: usize,
}

impl SearchHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            queries: Vec::new(),
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

        let file_path = config_dir.join("search_history.json");

        match file_path.exists() {
            true => {
                let content =
                    fs::read_to_string(&file_path).context("Failed to read search history file")?;
                let mut history: SearchHistory = serde_json::from_str(&content)
                    .context("Failed to parse search history file")?;
                history.file_path = Some(file_path);
                history.max_size = max_size;
                Ok(history)
            }
            false => Ok(Self {
                queries: Vec::new(),
                file_path: Some(file_path),
                max_size,
            }),
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = &self.file_path {
            let content =
                serde_json::to_string_pretty(self).context("Failed to serialize search history")?;
            fs::write(path, content).context("Failed to write search history file")?;
        }
        Ok(())
    }

    pub fn add(&mut self, query: String) {
        if query.is_empty() {
            return;
        }

        // Remove existing entry if present (to move it to top)
        self.queries.retain(|q| q != &query);

        self.queries.insert(0, query);

        // Enforce max size
        if self.queries.len() > self.max_size {
            self.queries.truncate(self.max_size);
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.queries.clear();
    }

    pub fn get_recent(&self, index: usize) -> Option<&String> {
        self.queries.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_mode_cycle() {
        assert_eq!(SearchMode::Title.next(), SearchMode::Comments);
        assert_eq!(SearchMode::Comments.next(), SearchMode::TitleAndComments);
        assert_eq!(SearchMode::TitleAndComments.next(), SearchMode::Title);
    }

    #[test]
    fn test_search_type_toggle() {
        assert_eq!(SearchType::Literal.toggle(), SearchType::Regex);
        assert_eq!(SearchType::Regex.toggle(), SearchType::Literal);
    }

    #[test]
    fn test_search_query_literal() {
        let query = SearchQuery::new("rust".to_string(), SearchMode::Title, SearchType::Literal);
        assert!(query.matches("Learning Rust"));
        assert!(query.matches("rust programming"));
        assert!(!query.matches("Python"));
    }

    #[test]
    fn test_search_query_regex() {
        let query = SearchQuery::new(
            "rust.*async".to_string(),
            SearchMode::Title,
            SearchType::Regex,
        );
        assert!(query.matches("rust with async"));
        assert!(query.matches("rust async"));
        assert!(!query.matches("rust programming"));
    }

    #[test]
    fn test_search_query_regex_error() {
        let query = SearchQuery::new("[invalid".to_string(), SearchMode::Title, SearchType::Regex);
        assert!(query.regex_error.is_some());
        assert!(!query.matches("anything"));
    }

    #[test]
    fn test_search_history_add() {
        let mut history = SearchHistory::new(5);
        history.add("rust".to_string());
        history.add("python".to_string());
        assert_eq!(history.queries.len(), 2);
        assert_eq!(history.queries[0], "python");
        assert_eq!(history.queries[1], "rust");

        // Add duplicate - should move to top
        history.add("rust".to_string());
        assert_eq!(history.queries.len(), 2);
        assert_eq!(history.queries[0], "rust");
    }

    #[test]
    fn test_search_history_max_size() {
        let mut history = SearchHistory::new(3);
        history.add("query1".to_string());
        history.add("query2".to_string());
        history.add("query3".to_string());
        history.add("query4".to_string());

        assert_eq!(history.queries.len(), 3);
        assert_eq!(history.queries[0], "query4");
        assert_eq!(history.queries[2], "query2");
    }
}

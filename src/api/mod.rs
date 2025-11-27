use crate::internal::cache::Cache;
use crate::internal::models::{Comment, Story};
use anyhow::{Context, Result};
use html2text::from_read; // Added for fetch_article_content
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;
use strum_macros::Display; // Added for fetch_article_content

/// Types of Hacker News story lists we can fetch.
#[derive(Debug, Clone, Copy, PartialEq, Display)]
#[allow(dead_code)]
pub enum StoryListType {
    Best,
    Top,
    New,
    Ask,
    Show,
    Job,
}

impl StoryListType {
    fn as_api_str(&self) -> &str {
        match self {
            Self::Best => "beststories",
            Self::Top => "topstories",
            Self::New => "newstories",
            Self::Ask => "askstories",
            Self::Show => "showstories",
            Self::Job => "jobstories",
        }
    }
}

const HN_API_BASE_URL: &str = "https://hacker-news.firebaseio.com/v0/";

#[cfg(test)]
pub fn hn_item_url(id: u32) -> String {
    format!("{}item/{}.json", HN_API_BASE_URL, id)
}

#[cfg(test)]
pub fn get_story_list_url(list_type: StoryListType) -> String {
    format!("{}{}.json", HN_API_BASE_URL, list_type.as_api_str())
}

/// HTTP API service for fetching Hacker News data.
///
/// This service uses `reqwest::blocking::Client` and returns `anyhow::Result` with
/// contextualized errors to preserve diagnostic information instead of erasing it
/// into plain strings.
#[derive(Clone)]
pub struct ApiService {
    client: Client,
    story_cache: Cache<u32, Story>,
    comment_cache: Cache<u32, Comment>,
    article_cache: Cache<String, String>,
    #[cfg(test)]
    base_url: Option<String>,
}

impl ApiService {
    /// Create a new `ApiService` with a default reqwest blocking client.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            story_cache: Cache::new(Duration::from_secs(300)), // 5 minutes
            comment_cache: Cache::new(Duration::from_secs(300)), // 5 minutes
            article_cache: Cache::new(Duration::from_secs(900)), // 15 minutes
            #[cfg(test)]
            base_url: None,
        }
    }

    #[cfg(test)]
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            story_cache: Cache::new(Duration::from_secs(300)),
            comment_cache: Cache::new(Duration::from_secs(300)),
            article_cache: Cache::new(Duration::from_secs(900)),
            base_url: Some(base_url),
        }
    }

    #[cfg(test)]
    fn get_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or(HN_API_BASE_URL)
    }

    #[cfg(not(test))]
    fn get_base_url(&self) -> &str {
        HN_API_BASE_URL
    }

    /// Generic helper to GET a URL and deserialize the JSON body into `T`.
    fn get_json<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let resp = self
            .client
            .get(url)
            .send()
            .with_context(|| format!("failed to send GET request to {}", url))?;

        resp.json::<T>()
            .with_context(|| format!("failed to parse JSON response from {}", url))
    }

    /// Fetch a list of story IDs for the given list type (e.g., top, new).
    pub fn fetch_story_ids(&self, list_type: StoryListType) -> Result<Vec<u32>> {
        let url = format!("{}{}.json", self.get_base_url(), list_type.as_api_str());
        self.get_json(&url)
            .with_context(|| format!("fetch_story_ids failed for list {:?}", list_type))
    }

    /// Fetch a single story item by id.
    pub fn fetch_story_content(&self, id: u32) -> Result<Story> {
        // Check cache first
        if let Some(story) = self.story_cache.get(&id) {
            return Ok(story);
        }

        // Fetch from API
        let url = format!("{}item/{}.json", self.get_base_url(), id);
        let story: Story = self
            .get_json(&url)
            .with_context(|| format!("fetch_story_content failed for id {}", id))?;

        // Cache the result
        self.story_cache.set(id, story.clone());
        Ok(story)
    }

    /// Fetch a single comment item by id.
    pub fn fetch_comment_content(&self, id: u32) -> Result<Comment> {
        // Check cache first
        if let Some(comment) = self.comment_cache.get(&id) {
            return Ok(comment);
        }

        // Fetch from API
        let url = format!("{}item/{}.json", self.get_base_url(), id);
        let comment: Comment = self
            .get_json(&url)
            .with_context(|| format!("fetch_comment_content failed for id {}", id))?;

        // Cache the result
        self.comment_cache.set(id, comment.clone());
        Ok(comment)
    }

    pub fn fetch_article_content(&self, url: &str) -> Result<String> {
        // Check cache first
        if let Some(content) = self.article_cache.get(&url.to_string()) {
            return Ok(content);
        }

        // Fetch from web
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .context("Failed to fetch article")?;

        let bytes = response.bytes().context("Failed to get response bytes")?;
        let text = from_read(&bytes[..], 80).context("Failed to convert HTML to text")?;

        // Cache the result
        self.article_cache.set(url.to_string(), text.clone());
        Ok(text)
    }
}

impl Default for ApiService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_list_type_as_api_str() {
        assert_eq!(StoryListType::Best.as_api_str(), "beststories");
        assert_eq!(StoryListType::Top.as_api_str(), "topstories");
        assert_eq!(StoryListType::New.as_api_str(), "newstories");
        assert_eq!(StoryListType::Ask.as_api_str(), "askstories");
        assert_eq!(StoryListType::Show.as_api_str(), "showstories");
        assert_eq!(StoryListType::Job.as_api_str(), "jobstories");
    }

    #[test]
    fn test_hn_item_url() {
        assert_eq!(
            hn_item_url(12345),
            "https://hacker-news.firebaseio.com/v0/item/12345.json"
        );
    }

    #[test]
    fn test_get_story_list_url() {
        assert_eq!(
            get_story_list_url(StoryListType::Top),
            "https://hacker-news.firebaseio.com/v0/topstories.json"
        );
        assert_eq!(
            get_story_list_url(StoryListType::Best),
            "https://hacker-news.firebaseio.com/v0/beststories.json"
        );
    }

    #[test]
    fn test_fetch_story_ids_success() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[1, 2, 3, 4, 5]")
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_story_ids(StoryListType::Top);

        mock.assert();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_fetch_story_ids_network_error() {
        // Use a URL that will fail to connect
        let service = ApiService::with_base_url("http://localhost:1/".to_string());
        let result = service.fetch_story_ids(StoryListType::Top);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("fetch_story_ids failed"));
    }

    #[test]
    fn test_fetch_story_content_success() {
        let mut server = mockito::Server::new();
        let story_json = r#"{
            "by": "testuser",
            "descendants": 10,
            "id": 12345,
            "kids": [1, 2, 3],
            "score": 100,
            "time": 1234567890,
            "title": "Test Story",
            "type": "story",
            "url": "https://example.com"
        }"#;

        let mock = server
            .mock("GET", "/item/12345.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(story_json)
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_story_content(12345);

        mock.assert();
        assert!(result.is_ok());
        let story = result.unwrap();
        assert_eq!(story.id, 12345);
        assert_eq!(story.title, Some("Test Story".to_string()));
        assert_eq!(story.by, Some("testuser".to_string()));
    }

    #[test]
    fn test_fetch_story_content_invalid_json() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/item/12345.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("invalid json")
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_story_content(12345);

        mock.assert();
        assert!(result.is_err());
        // Just verify we got an error - the exact error message may vary
    }

    #[test]
    fn test_fetch_comment_content_success() {
        let mut server = mockito::Server::new();
        let comment_json = r#"{
            "by": "commenter",
            "id": 67890,
            "kids": [10, 11],
            "parent": 12345,
            "text": "This is a comment",
            "time": 1234567890,
            "type": "comment"
        }"#;

        let mock = server
            .mock("GET", "/item/67890.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(comment_json)
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_comment_content(67890);

        mock.assert();
        assert!(result.is_ok());
        let comment = result.unwrap();
        assert_eq!(comment.id, 67890);
        assert_eq!(comment.by, Some("commenter".to_string()));
        assert_eq!(comment.text, Some("This is a comment".to_string()));
    }

    #[test]
    fn test_fetch_comment_content_http_error() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/item/99999.json")
            .with_status(404)
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_comment_content(99999);

        mock.assert();
        // reqwest will succeed on 404 but JSON parsing should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_api_service_default() {
        let service = ApiService::default();
        // Just verify we can create a default instance
        assert!(service.client.get("https://example.com").build().is_ok());
    }
}

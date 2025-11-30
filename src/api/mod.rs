use crate::internal::cache::Cache;
use crate::internal::models::{Article, Comment, Story};
use crate::utils::html_parser::parse_article_html;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use strum_macros::Display;

/// Types of Hacker News story lists we can fetch.
#[derive(Debug, Clone, Copy, PartialEq, Display, Serialize, Deserialize)]
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
/// This service uses async `reqwest::Client` and returns `anyhow::Result` with
/// contextualized errors to preserve diagnostic information instead of erasing it
/// into plain strings.
#[derive(Clone)]
pub struct ApiService {
    client: Client,
    story_cache: Cache<u32, Story>,
    comment_cache: Cache<u32, Comment>,
    article_cache: Cache<String, Article>,
    network_config: crate::config::NetworkConfig,
    enable_performance_metrics: bool,
    // Exposed for integration tests
    pub base_url: Option<String>,
}

impl ApiService {
    /// Create a new `ApiService` with a default async reqwest client.
    pub fn new(
        network_config: crate::config::NetworkConfig,
        enable_performance_metrics: bool,
    ) -> Self {
        Self {
            client: Client::new(),
            story_cache: Cache::with_metrics(Duration::from_secs(300), enable_performance_metrics), // 5 minutes
            comment_cache: Cache::with_metrics(
                Duration::from_secs(300),
                enable_performance_metrics,
            ), // 5 minutes
            article_cache: Cache::with_metrics(
                Duration::from_secs(900),
                enable_performance_metrics,
            ), // 15 minutes
            network_config,
            enable_performance_metrics,
            base_url: None,
        }
    }

    /// Helper to create a service with a custom base URL (for testing).
    #[allow(dead_code)]
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            story_cache: Cache::with_metrics(Duration::from_secs(300), false),
            comment_cache: Cache::with_metrics(Duration::from_secs(300), false),
            article_cache: Cache::with_metrics(Duration::from_secs(900), false),
            network_config: crate::config::NetworkConfig::default(),
            enable_performance_metrics: false,
            base_url: Some(base_url),
        }
    }

    fn get_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or(HN_API_BASE_URL)
    }

    /// Generic helper to GET a URL and deserialize the JSON body into `T`.
    /// Retries on network errors and timeouts with exponential backoff.
    #[tracing::instrument(skip(self), fields(url = %url))]
    async fn get_json<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let start = std::time::Instant::now();
        let mut attempt = 0;
        let mut delay = self.network_config.initial_retry_delay_ms;

        loop {
            attempt += 1;
            let resp_result = self.client.get(url).send().await;

            match resp_result {
                Ok(resp) => {
                    // Successful connection: parse JSON
                    let parsed = resp
                        .json::<T>()
                        .await
                        .with_context(|| format!("failed to parse JSON response from {}", url))?;
                    if self.enable_performance_metrics {
                        tracing::debug!(elapsed = ?start.elapsed(), url = %url, attempt = attempt, "GET JSON successful");
                    }
                    return Ok(parsed);
                }
                Err(e) => {
                    // Check if we should retry
                    let is_timeout = e.is_timeout();
                    let is_connect = e.is_connect();
                    let should_retry =
                        (is_timeout && self.network_config.retry_on_timeout) || is_connect;

                    if !should_retry || attempt > self.network_config.max_retries {
                        if self.enable_performance_metrics {
                            tracing::debug!(elapsed = ?start.elapsed(), url = %url, attempt = attempt, error = %e, "GET JSON failed (final)");
                        }
                        return Err(anyhow::Error::new(e))
                            .with_context(|| format!("failed to send GET request to {}", url));
                    }

                    tracing::warn!(
                        "Request to {} failed (attempt {}/{}): {}. Retrying in {}ms...",
                        url,
                        attempt,
                        self.network_config.max_retries + 1,
                        e,
                        delay
                    );

                    // Wait before retrying (async sleep)
                    tokio::time::sleep(Duration::from_millis(delay)).await;

                    // Exponential backoff with cap
                    delay = (delay * 2).min(self.network_config.max_retry_delay_ms);
                }
            }
        }
    }

    /// Fetch a list of story IDs for the given list type (e.g., top, new).
    #[tracing::instrument(skip(self), fields(list_type = ?list_type))]
    pub async fn fetch_story_ids(&self, list_type: StoryListType) -> Result<Vec<u32>> {
        let start = std::time::Instant::now();
        let url = format!("{}{}.json", self.get_base_url(), list_type.as_api_str());
        let result: Vec<u32> = self
            .get_json(&url)
            .await
            .with_context(|| format!("fetch_story_ids failed for list {:?}", list_type))?;

        if self.enable_performance_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), count = result.len(), "Fetched story IDs");
        }
        Ok(result)
    }

    /// Fetch a single story item by id.
    #[tracing::instrument(skip(self), fields(id = %id))]
    pub async fn fetch_story_content(&self, id: u32) -> Result<Story> {
        // Check cache first
        if let Some(story) = self.story_cache.get(&id) {
            tracing::trace!("Cache hit for story {}", id);
            return Ok(story);
        }

        if self.enable_performance_metrics {
            tracing::trace!("Cache miss for story {}", id);
        }

        let start = std::time::Instant::now();
        // Fetch from API
        let url = format!("{}item/{}.json", self.get_base_url(), id);
        let story: Story = self
            .get_json(&url)
            .await
            .with_context(|| format!("fetch_story_content failed for id {}", id))?;

        // Cache the result
        self.story_cache.set(id, story.clone());
        if self.enable_performance_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), "Fetched and cached story content");
        }
        Ok(story)
    }

    /// Fetch a single comment item by id.
    #[tracing::instrument(skip(self), fields(id = %id))]
    pub async fn fetch_comment_content(&self, id: u32) -> Result<Comment> {
        // Check cache first
        if let Some(comment) = self.comment_cache.get(&id) {
            tracing::trace!("Cache hit for comment {}", id);
            return Ok(comment);
        }

        if self.enable_performance_metrics {
            tracing::trace!("Cache miss for comment {}", id);
        }

        let start = std::time::Instant::now();
        // Fetch from API
        let url = format!("{}item/{}.json", self.get_base_url(), id);
        let comment: Comment = self
            .get_json(&url)
            .await
            .with_context(|| format!("fetch_comment_content failed for id {}", id))?;

        // Cache the result
        self.comment_cache.set(id, comment.clone());
        if self.enable_performance_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), "Fetched and cached comment content");
        }
        Ok(comment)
    }

    /// Fetch a tree of comments starting from the given root IDs.
    /// Returns a flattened list of CommentRows in DFS order.
    #[tracing::instrument(skip(self, root_ids), fields(root_count = root_ids.len()))]
    pub async fn fetch_comment_tree(
        &self,
        root_ids: Vec<u32>,
    ) -> Result<Vec<crate::internal::models::CommentRow>> {
        let start = std::time::Instant::now();
        let mut rows = Vec::new();
        // Limit total comments to prevent freezing on huge threads for now
        let mut total_fetched = 0;
        const MAX_COMMENTS: usize = 100;

        for id in root_ids {
            if total_fetched >= MAX_COMMENTS {
                break;
            }
            self.fetch_comment_recursive(id, 0, None, &mut rows, &mut total_fetched)
                .await?;
        }
        if self.enable_performance_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), fetched = total_fetched, "Fetched comment tree");
        }
        Ok(rows)
    }

    fn fetch_comment_recursive<'a>(
        &'a self,
        id: u32,
        depth: usize,
        parent_id: Option<u32>,
        rows: &'a mut Vec<crate::internal::models::CommentRow>,
        total_fetched: &'a mut usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if *total_fetched >= 100 {
                return Ok(());
            }

            // Fetch the comment (uses cache internally)
            match self.fetch_comment_content(id).await {
                Ok(comment) => {
                    *total_fetched += 1;
                    let kids = comment.kids.clone();

                    rows.push(crate::internal::models::CommentRow {
                        comment,
                        depth,
                        expanded: true,
                        parent_id,
                    });

                    if let Some(kids) = kids {
                        for kid_id in kids {
                            self.fetch_comment_recursive(
                                kid_id,
                                depth + 1,
                                Some(id),
                                rows,
                                total_fetched,
                            )
                            .await?;
                        }
                    }
                }
                Err(_) => {
                    // If a comment fails to load, just skip it and its children
                }
            }
            Ok(())
        })
    }

    #[tracing::instrument(skip(self), fields(url = %url))]
    pub async fn fetch_article_content(&self, url: &str) -> Result<Article> {
        // Check cache first
        if let Some(article) = self.article_cache.get(&url.to_string()) {
            tracing::trace!("Cache hit for article {}", url);
            return Ok(article);
        }

        if self.enable_performance_metrics {
            tracing::trace!("Cache miss for article {}", url);
        }

        let start = std::time::Instant::now();
        // Fetch from web
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .context("Failed to fetch article")?;

        let html = response
            .text()
            .await
            .context("Failed to get response text")?;
        let elements = parse_article_html(&html);

        // Simple title extraction heuristic (could be improved)
        let title = "Article".to_string();

        let article = Article { title, elements };

        // Cache the result
        self.article_cache.set(url.to_string(), article.clone());
        if self.enable_performance_metrics {
            tracing::debug!(elapsed = ?start.elapsed(), "Fetched and cached article content");
        }
        Ok(article)
    }
}

impl Default for ApiService {
    fn default() -> Self {
        Self::new(crate::config::NetworkConfig::default(), false)
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

    #[tokio::test]
    async fn test_fetch_story_ids_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[1, 2, 3, 4, 5]")
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_story_ids(StoryListType::Top).await;

        mock.assert();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_fetch_story_ids_network_error() {
        // Use a URL that will fail to connect
        let service = ApiService::with_base_url("http://localhost:1/".to_string());
        let result = service.fetch_story_ids(StoryListType::Top).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("fetch_story_ids failed"));
    }

    #[tokio::test]
    async fn test_fetch_story_content_success() {
        let mut server = mockito::Server::new_async().await;
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
        let result = service.fetch_story_content(12345).await;

        mock.assert();
        assert!(result.is_ok());
        let story = result.unwrap();
        assert_eq!(story.id, 12345);
        assert_eq!(story.title, Some("Test Story".to_string()));
        assert_eq!(story.by, Some("testuser".to_string()));
    }

    #[tokio::test]
    async fn test_fetch_story_content_invalid_json() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/item/12345.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("invalid json")
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_story_content(12345).await;

        mock.assert();
        assert!(result.is_err());
        // Just verify we got an error - the exact error message may vary
    }

    #[tokio::test]
    async fn test_fetch_comment_content_success() {
        let mut server = mockito::Server::new_async().await;
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
        let result = service.fetch_comment_content(67890).await;

        mock.assert();
        assert!(result.is_ok());
        let comment = result.unwrap();
        assert_eq!(comment.id, 67890);
        assert_eq!(comment.by, Some("commenter".to_string()));
        assert_eq!(comment.text, Some("This is a comment".to_string()));
    }

    #[tokio::test]
    async fn test_fetch_comment_content_http_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/item/99999.json")
            .with_status(404)
            .create();

        let service = ApiService::with_base_url(format!("{}/", server.url()));
        let result = service.fetch_comment_content(99999).await;

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

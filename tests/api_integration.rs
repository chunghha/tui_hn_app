use tui_hn_app::api::{ApiService, StoryListType};

#[test]
fn test_integration_fetch_top_stories() {
    let mut server = mockito::Server::new();
    let _m = server
        .mock("GET", "/topstories.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[1001, 1002, 1003]")
        .create();

    let service = ApiService::with_base_url(format!("{}/", server.url()));
    let stories = service
        .fetch_story_ids(StoryListType::Top)
        .expect("Failed to fetch stories");

    assert_eq!(stories, vec![1001, 1002, 1003]);
}

#[test]
fn test_integration_fetch_story_details() {
    let mut server = mockito::Server::new();
    let story_json = r#"{
        "id": 2001,
        "title": "Integration Test Story",
        "by": "tester",
        "score": 42,
        "time": 1600000000,
        "type": "story",
        "url": "https://example.com"
    }"#;

    let _m = server
        .mock("GET", "/item/2001.json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(story_json)
        .create();

    let service = ApiService::with_base_url(format!("{}/", server.url()));
    let story = service
        .fetch_story_content(2001)
        .expect("Failed to fetch story");

    assert_eq!(story.id, 2001);
    assert_eq!(story.title.unwrap(), "Integration Test Story");
    assert_eq!(story.by.unwrap(), "tester");
}

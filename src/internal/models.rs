use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Story {
    pub id: u32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub by: Option<String>,
    pub score: Option<u32>,
    pub time: Option<i64>,
    pub descendants: Option<u32>,
    pub kids: Option<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub enum FetchState {
    #[default]
    Idle,
    Loading,
    Failed,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Comment {
    pub id: u32,
    pub by: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub time: Option<i64>,
    pub kids: Option<Vec<u32>>,
    #[serde(default)]
    pub deleted: bool,
}

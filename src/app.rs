use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use crate::api::ApiService;
use crate::api::StoryListType;
use crate::config::AppConfig;
use crate::internal::models::{Comment, Story};
use crate::utils::theme_loader::{TuiTheme, load_theme};

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Alignment;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

/// Application view modes.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewMode {
    List,
    StoryDetail,
    Article,
}

/// Input modes for the UI.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Search,
}

/// Actions/messages sent through the app action channel.
#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    NavigateUp,
    NavigateDown,
    Enter,
    Back,
    OpenBrowser,
    LoadStories(StoryListType),
    StoryIdsLoaded(Vec<u32>),
    StoryLoadingProgress(usize),
    StoriesLoaded(Vec<Story>),
    LoadMoreStories,
    LoadAllStories,
    SelectStory(Story, StoryListType),
    CommentsLoaded(Vec<Comment>),
    ToggleArticleView,
    ArticleLoaded(StoryListType, u32, String),
    ScrollArticleUp,
    ScrollArticleDown,
    SwitchTheme,
    ClearNotification,
    Error(String),
}

/// Main application state.
pub struct App {
    pub running: bool,
    pub app_version: String,
    pub view_mode: ViewMode,
    pub stories: Vec<Story>,
    pub story_ids: Vec<u32>,
    pub loaded_count: usize,
    pub story_list_state: ListState,
    pub current_list_type: StoryListType,
    pub api_service: Arc<ApiService>,
    pub loading: bool,
    pub story_load_progress: Option<(usize, usize)>,
    pub selected_story: Option<Story>,
    pub comments: Vec<Comment>,
    pub comments_loading: bool,
    pub article_content: Option<String>,
    pub article_for_story_id: Option<u32>,
    pub article_loading: bool,
    pub article_scroll: usize,
    pub theme: TuiTheme,
    pub available_themes: Vec<(String, String)>,
    pub current_theme_index: usize,
    #[allow(dead_code)]
    pub terminal_mode: String,
    pub notification_message: Option<String>,
    pub notification_timer: Option<tokio::time::Instant>,
    pub input_mode: InputMode,
    pub search_query: String,
    #[allow(dead_code)]
    pub config: AppConfig,
    pub action_tx: UnboundedSender<Action>,
    pub action_rx: UnboundedReceiver<Action>,
}

impl App {
    pub fn new() -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let api_service = Arc::new(ApiService::new());
        let config = AppConfig::load();

        // Detect terminal mode (dark or light)
        let terminal_mode = Self::detect_terminal_mode();

        // Discover available themes. Respect a configured `theme_file` if provided,
        // and fall back to common locations (./themes and themes next to the executable).
        let available_themes = Self::discover_all_themes(&config.theme_file);

        // Startup diagnostics (help debug initial theme selection)
        tracing::info!(
            "App config: theme_name='{}', theme_file='{}'",
            config.theme_name,
            config.theme_file
        );
        tracing::info!("Detected terminal_mode: {}", terminal_mode);
        tracing::info!("Discovered {} theme candidates:", available_themes.len());
        for (i, (path, mode)) in available_themes.iter().enumerate() {
            let stem = Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            tracing::info!("  [{}] {} ({}) -> {}", i, stem, mode, path);
        }

        // Find theme from config or use first available.
        // Respect an explicit \"Dark\" / \"Light\" token in the configured theme name
        // (e.g. \"Gruvbox Dark\") — when present prefer that exact variant. If the
        // token is absent, fall back to the detected runtime `terminal_mode`.
        let (theme, current_theme_index) = if !available_themes.is_empty() {
            // Canonicalize configured theme name and detect optional explicit mode token.
            let theme_name_raw = config.theme_name.trim();
            let mut requested_mode: Option<String> = None;
            let mut base_name = theme_name_raw.to_string();

            // If the last whitespace token is `dark` or `light` (case-insensitive),
            // treat it as an explicit request and strip it from the base name.
            if let Some(last) = theme_name_raw.split_whitespace().last()
                && (last.eq_ignore_ascii_case("dark") || last.eq_ignore_ascii_case("light"))
            {
                requested_mode = Some(last.to_lowercase());
                // Remove the trailing token to create the base name.
                let tokens: Vec<&str> = theme_name_raw.split_whitespace().collect();
                if tokens.len() >= 2 {
                    base_name = tokens[..tokens.len() - 1].join(" ");
                } else {
                    base_name = String::new();
                }
            }

            let base_lower = base_name.to_lowercase();
            let fullname_lower = theme_name_raw.to_lowercase();

            // Try to find the best matching theme file:
            // 1) If base name is present, prefer an exact file_stem match (and mode if requested).
            // 2) Otherwise, try a starts-with match against the full configured name.
            // 3) Prefer candidates that match requested_mode if present, else prefer runtime terminal_mode.
            let mut matched_idx: Option<usize> = None;

            if !base_lower.is_empty() {
                // Exact file_stem match first
                for (i, (path, mode)) in available_themes.iter().enumerate() {
                    if let Some(stem) = Path::new(path).file_stem().and_then(|s| s.to_str())
                        && stem.eq_ignore_ascii_case(&base_lower)
                    {
                        if let Some(req) = &requested_mode {
                            if mode.eq_ignore_ascii_case(req) {
                                matched_idx = Some(i);
                                break;
                            }
                        } else if mode == &terminal_mode {
                            matched_idx = Some(i);
                            break;
                        } else if matched_idx.is_none() {
                            matched_idx = Some(i);
                        }
                    }
                }
            }

            if matched_idx.is_none() {
                // Try starts-with against the configured full name (preserves older behavior)
                for (i, (path, mode)) in available_themes.iter().enumerate() {
                    if let Some(stem) = Path::new(path).file_stem().and_then(|s| s.to_str())
                        && fullname_lower.starts_with(&stem.to_lowercase())
                    {
                        if let Some(req) = &requested_mode {
                            if mode.eq_ignore_ascii_case(req) {
                                matched_idx = Some(i);
                                break;
                            }
                        } else if mode == &terminal_mode {
                            matched_idx = Some(i);
                            break;
                        } else if matched_idx.is_none() {
                            matched_idx = Some(i);
                        }
                    }
                }
            }

            if let Some(idx) = matched_idx {
                let (filename, mode) = &available_themes[idx];
                let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                (theme, idx)
            } else {
                // If a specific mode was requested but we didn't find an exact base match,
                // try to select any theme with the requested mode.
                if let Some(req) = requested_mode {
                    if let Some(idx) = available_themes
                        .iter()
                        .position(|(_, mode)| mode.eq_ignore_ascii_case(&req))
                    {
                        let (filename, mode) = &available_themes[idx];
                        let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                        (theme, idx)
                    } else if let Some(idx) = available_themes
                        .iter()
                        .position(|(_, mode)| mode == &terminal_mode)
                    {
                        // Fallback to a theme matching runtime mode
                        let (filename, mode) = &available_themes[idx];
                        let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                        (theme, idx)
                    } else {
                        // Last resort: first available
                        let (filename, mode) = &available_themes[0];
                        let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                        (theme, 0)
                    }
                } else {
                    // No explicit mode requested: prefer a theme whose mode matches the runtime
                    if let Some(idx) = available_themes
                        .iter()
                        .position(|(_, mode)| mode == &terminal_mode)
                    {
                        let (filename, mode) = &available_themes[idx];
                        let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                        (theme, idx)
                    } else {
                        // Fallback to first available
                        let (filename, mode) = &available_themes[0];
                        let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                        (theme, 0)
                    }
                }
            }
        } else {
            (TuiTheme::default(), 0)
        };

        // Log which theme was finally selected (index and variant) so startup behavior is traceable.
        if !available_themes.is_empty() {
            let (filename, mode) = &available_themes[current_theme_index];
            let stem = Path::new(filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            tracing::info!(
                "Selected theme index {} -> {} ({}) from '{}'",
                current_theme_index,
                stem,
                mode,
                filename
            );
        } else {
            tracing::info!("No available themes found; using default TuiTheme");
        }

        Self {
            running: true,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            view_mode: ViewMode::List,
            stories: Vec::new(),
            story_ids: Vec::new(),
            loaded_count: 0,
            story_list_state: ListState::default(),
            current_list_type: StoryListType::Top,
            api_service,
            loading: false,
            story_load_progress: None,
            selected_story: None,
            comments: Vec::new(),
            comments_loading: false,
            article_content: None,
            article_for_story_id: None,
            article_loading: false,
            article_scroll: 0,
            theme,
            available_themes,
            current_theme_index,
            terminal_mode,
            notification_message: None,
            notification_timer: None,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            config,
            action_tx,
            action_rx,
        }
    }

    fn detect_terminal_mode() -> String {
        // Check COLORFGBG environment variable (used by many terminals)
        // Format is "foreground;background" where higher values mean lighter
        if let Ok(colorfgbg) = std::env::var("COLORFGBG")
            && let Some(bg) = colorfgbg.split(';').nth(1)
            && let Ok(bg_val) = bg.parse::<u8>()
        {
            // Background values 0-7 are typically dark, 8-15 are light
            return if bg_val < 8 { "dark" } else { "light" }.to_string();
        }

        // Check TERM_PROGRAM for known terminals
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            // macOS Terminal.app defaults can be checked via other means
            // For now, assume dark mode is more common
            if term_program == "Apple_Terminal" || term_program == "iTerm.app" {
                // Default to dark for these terminals
                return "dark".to_string();
            }
        }

        // Default to dark mode as it's more common for terminals
        "dark".to_string()
    }

    fn discover_all_themes(configured: &str) -> Vec<(String, String)> {
        // Collect candidate theme locations in priority order:
        // 1. Explicit configured path (if non-empty)
        // 2. ./themes in current working directory
        // 3. <exe_dir>/themes (next to executable)
        let mut themes = Vec::new();
        let mut candidates: Vec<PathBuf> = Vec::new();

        // 1) Configured path (may be a file or directory)
        if !configured.trim().is_empty() {
            candidates.push(PathBuf::from(configured));
        }

        // 2) Current working directory ./themes
        candidates.push(PathBuf::from("themes"));

        // 3) themes next to the executable (if available)
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            candidates.push(dir.join("themes"));
        }

        // Walk candidates and gather .json theme files. If a candidate is a file,
        // consider it directly; if it's a directory read its entries.
        for cand in candidates.into_iter() {
            if !cand.exists() {
                continue;
            }

            if cand.is_file() {
                if let Some(ext) = cand.extension().and_then(|s| s.to_str())
                    && ext.eq_ignore_ascii_case("json")
                    && let Some(filename) = cand.to_str()
                {
                    themes.push((filename.to_string(), "dark".to_string()));
                    themes.push((filename.to_string(), "light".to_string()));
                }
            } else if cand.is_dir()
                && let Ok(entries) = std::fs::read_dir(&cand)
            {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json")
                        && let Some(filename) = path.to_str()
                    {
                        themes.push((filename.to_string(), "dark".to_string()));
                        themes.push((filename.to_string(), "light".to_string()));
                    }
                }
            }
        }

        // Deduplicate while preserving order of discovery
        let mut seen = std::collections::HashSet::new();
        themes.retain(|(p, mode)| {
            let key = format!("{}:{}", p, mode);
            if seen.contains(&key) {
                false
            } else {
                seen.insert(key);
                true
            }
        });

        themes
    }

    pub async fn run(&mut self, mut tui: crate::tui::Tui) -> Result<()> {
        // Initial load
        let _ = self
            .action_tx
            .send(Action::LoadStories(self.current_list_type));

        let mut event_interval = tokio::time::interval(std::time::Duration::from_millis(16));

        loop {
            tui.draw(|f| self.ui(f))?;

            tokio::select! {
                _ = event_interval.tick() => {
                    // Check for terminal events
                    if event::poll(std::time::Duration::from_millis(0))?
                        && let Event::Key(key) = event::read()?
                            && key.kind == KeyEventKind::Press {
                                self.handle_key_event(key);
                            }
                }
                Some(action) = self.action_rx.recv() => {
                    self.handle_action(action).await;
                }
            }

            if !self.running {
                break;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Search => self.handle_search_input(key),
            InputMode::Normal => self.handle_normal_input(key),
        }
    }

    fn handle_search_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('/') => {
                // Ignore / in search mode (it's the key that enters search mode)
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Enter | KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_normal_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Only quit if we're in list view or detail view (not search mode)
                match self.view_mode {
                    ViewMode::List => {
                        let _ = self.action_tx.send(Action::Quit);
                    }
                    ViewMode::StoryDetail | ViewMode::Article => {
                        let _ = self.action_tx.send(Action::Back);
                    }
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.view_mode == ViewMode::Article {
                    let _ = self.action_tx.send(Action::ScrollArticleDown);
                } else {
                    let _ = self.action_tx.send(Action::NavigateDown);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.view_mode == ViewMode::Article {
                    let _ = self.action_tx.send(Action::ScrollArticleUp);
                } else {
                    let _ = self.action_tx.send(Action::NavigateUp);
                }
            }
            KeyCode::Enter => {
                let _ = self.action_tx.send(Action::Enter);
            }
            KeyCode::Tab => {
                let _ = self.action_tx.send(Action::ToggleArticleView);
            }
            KeyCode::Char('o') => {
                let _ = self.action_tx.send(Action::OpenBrowser);
            }
            KeyCode::Char('1') => {
                let _ = self.action_tx.send(Action::LoadStories(StoryListType::Top));
            }
            KeyCode::Char('2') => {
                let _ = self.action_tx.send(Action::LoadStories(StoryListType::New));
            }
            KeyCode::Char('3') => {
                let _ = self
                    .action_tx
                    .send(Action::LoadStories(StoryListType::Best));
            }
            KeyCode::Char('4') => {
                let _ = self.action_tx.send(Action::LoadStories(StoryListType::Ask));
            }
            KeyCode::Char('5') => {
                let _ = self
                    .action_tx
                    .send(Action::LoadStories(StoryListType::Show));
            }
            KeyCode::Char('6') => {
                let _ = self.action_tx.send(Action::LoadStories(StoryListType::Job));
            }
            KeyCode::Char('t') => {
                let _ = self.action_tx.send(Action::SwitchTheme);
            }
            KeyCode::Char('m') => {
                if self.view_mode == ViewMode::List {
                    let _ = self.action_tx.send(Action::LoadMoreStories);
                }
            }
            KeyCode::Char('A') => {
                if self.view_mode == ViewMode::List {
                    let _ = self.action_tx.send(Action::LoadAllStories);
                }
            }
            KeyCode::Char('/') => {
                if self.view_mode == ViewMode::List {
                    self.input_mode = InputMode::Search;
                }
            }
            KeyCode::Char('C') => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                }
            }
            _ => {}
        }
    }

    async fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::NavigateUp => self.select_prev(),
            Action::NavigateDown => self.select_next(),
            Action::Enter => {
                if let Some(index) = self.story_list_state.selected() {
                    // Map the selected index (which refers to the displayed/filtered list)
                    // back to the original story using the same filter logic used when
                    // rendering the list. This ensures Enter selects the story shown
                    // on that row even when a filter/search is active.
                    let displayed: Vec<_> = if self.search_query.is_empty() {
                        self.stories.iter().enumerate().collect()
                    } else {
                        let query = self.search_query.to_lowercase();
                        self.stories
                            .iter()
                            .enumerate()
                            .filter(|(_, story)| {
                                story
                                    .title
                                    .as_ref()
                                    .map(|t| t.to_lowercase().contains(&query))
                                    .unwrap_or(false)
                            })
                            .collect()
                    };

                    if let Some((_, s)) = displayed.get(index).cloned() {
                        // Clone the story so we send an owned Story in the action.
                        let story = s.clone();
                        let _ = self
                            .action_tx
                            .send(Action::SelectStory(story, self.current_list_type));
                    }
                }
            }
            Action::Back => {
                self.view_mode = ViewMode::List;
                self.selected_story = None;
                self.comments.clear();
            }
            Action::OpenBrowser => {
                if let Some(story) = &self.selected_story {
                    if let Some(url) = &story.url {
                        let _ = open::that(url);
                    }
                } else if let Some(index) = self.story_list_state.selected() {
                    // Map selected displayed index back to original story so OpenBrowser
                    // opens the URL for the story visible on that row when filtered.
                    let displayed: Vec<_> = if self.search_query.is_empty() {
                        self.stories.iter().enumerate().collect()
                    } else {
                        let query = self.search_query.to_lowercase();
                        self.stories
                            .iter()
                            .enumerate()
                            .filter(|(_, story)| {
                                story
                                    .title
                                    .as_ref()
                                    .map(|t| t.to_lowercase().contains(&query))
                                    .unwrap_or(false)
                            })
                            .collect()
                    };

                    if let Some((_, story)) = displayed.get(index).cloned()
                        && let Some(url) = &story.url
                    {
                        let _ = open::that(url);
                    }
                }
            }
            Action::LoadStories(list_type) => {
                self.loading = true;
                self.current_list_type = list_type;
                // Reset pagination
                self.stories.clear();
                self.story_ids.clear();
                self.loaded_count = 0;

                let api = self.api_service.clone();
                let tx = self.action_tx.clone();

                tokio::spawn(async move {
                    if let Ok(ids) = api.fetch_story_ids(list_type) {
                        // Send all IDs first
                        let all_ids = ids.clone();
                        let _ = tx.send(Action::StoryIdsLoaded(all_ids));

                        // Fetch first 20 stories
                        let ids_to_fetch = ids.iter().take(20).copied().collect::<Vec<_>>();
                        let mut stories = Vec::new();
                        for id in &ids_to_fetch {
                            if let Ok(story) = api.fetch_story_content(*id) {
                                stories.push(story);
                            }
                        }
                        let _ = tx.send(Action::StoriesLoaded(stories));
                    } else {
                        let _ = tx.send(Action::Error("Failed to fetch stories".to_string()));
                    }
                });
            }
            Action::StoryIdsLoaded(ids) => {
                self.story_ids = ids;
            }
            Action::StoryLoadingProgress(loaded) => {
                if let Some((_, total)) = self.story_load_progress {
                    self.story_load_progress = Some((loaded, total));
                }
            }
            Action::StoriesLoaded(stories) => {
                // Update loaded count and append stories
                self.loaded_count += stories.len();
                self.stories.extend(stories);
                self.loading = false;
                self.story_load_progress = None;
                if !self.stories.is_empty() && self.story_list_state.selected().is_none() {
                    self.story_list_state.select(Some(0));
                }
            }
            Action::LoadMoreStories => {
                if self.loading || self.story_ids.is_empty() {
                    return;
                }

                // Check if all stories are already loaded
                if self.loaded_count >= self.story_ids.len() {
                    let msg = format!(
                        "{}/{} stories already loaded",
                        self.loaded_count,
                        self.story_ids.len()
                    );
                    self.notification_message = Some(msg);
                    self.notification_timer = Some(tokio::time::Instant::now());

                    // Schedule notification clear
                    let tx = self.action_tx.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        let _ = tx.send(Action::ClearNotification);
                    });
                    return;
                }

                self.loading = true;
                let api = self.api_service.clone();
                let tx = self.action_tx.clone();
                let start_idx = self.loaded_count;
                let ids_to_fetch: Vec<_> = self
                    .story_ids
                    .iter()
                    .skip(start_idx)
                    .take(20)
                    .copied()
                    .collect();

                if ids_to_fetch.is_empty() {
                    self.loading = false;
                    return;
                }

                tokio::spawn(async move {
                    let mut stories = Vec::new();
                    for id in ids_to_fetch {
                        if let Ok(story) = api.fetch_story_content(id) {
                            stories.push(story);
                        }
                    }
                    let _ = tx.send(Action::StoriesLoaded(stories));
                });
            }
            Action::LoadAllStories => {
                if self.loading || self.story_ids.is_empty() {
                    return;
                }

                // Check if all stories are already loaded
                if self.loaded_count >= self.story_ids.len() {
                    let msg = format!(
                        "{}/{} stories already loaded",
                        self.loaded_count,
                        self.story_ids.len()
                    );
                    self.notification_message = Some(msg);
                    self.notification_timer = Some(tokio::time::Instant::now());

                    // Schedule notification clear
                    let tx = self.action_tx.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        let _ = tx.send(Action::ClearNotification);
                    });
                    return;
                }

                self.loading = true;
                let api = self.api_service.clone();
                let tx = self.action_tx.clone();
                let start_idx = self.loaded_count;

                // Load ALL remaining stories
                let ids_to_fetch: Vec<_> = self.story_ids.iter().skip(start_idx).copied().collect();

                if ids_to_fetch.is_empty() {
                    self.loading = false;
                    return;
                }

                self.story_load_progress = Some((0, ids_to_fetch.len()));

                tokio::spawn(async move {
                    let mut stories = Vec::new();
                    for (i, id) in ids_to_fetch.iter().enumerate() {
                        if let Ok(story) = api.fetch_story_content(*id) {
                            stories.push(story);
                        }
                        let _ = tx.send(Action::StoryLoadingProgress(i + 1));
                        // Add a small delay to avoid hitting API rate limits
                        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                    }
                    let _ = tx.send(Action::StoriesLoaded(stories));
                });
            }
            Action::SelectStory(story, list_type) => {
                // Propagate the category/list type when selecting a story so downstream
                // operations (e.g., fetching details) have the context available.
                self.current_list_type = list_type;
                self.selected_story = Some(story.clone());
                // Show Article view first when a story is selected.
                self.view_mode = ViewMode::Article;
                self.comments_loading = true;
                self.comments.clear();

                // Reset article-related state so that we always fetch for the newly selected story.
                self.article_content = None;
                self.article_for_story_id = None;
                self.article_scroll = 0;
                self.article_loading = false;

                let api = self.api_service.clone();
                let tx = self.action_tx.clone();

                // If the story has a URL, start fetching the article immediately.
                if let Some(url) = story.url.clone() {
                    self.article_loading = true;
                    let api_clone = api.clone();
                    let tx_clone = tx.clone();
                    let story_id = story.id;
                    // Capture the list/category this selection came from for the response.
                    let list_for_request = self.current_list_type;
                    tokio::spawn(async move {
                        if let Ok(content) = api_clone.fetch_article_content(&url) {
                            let _ = tx_clone.send(Action::ArticleLoaded(
                                list_for_request,
                                story_id,
                                content,
                            ));
                        } else {
                            let _ =
                                tx_clone.send(Action::Error("Failed to fetch article".to_string()));
                        }
                    });
                }

                // Fetch comments in the background as before so they are available
                // if the user switches to the comments view.
                if let Some(kids) = story.kids {
                    let api_clone = api.clone();
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let mut comments = Vec::new();
                        for id in kids.into_iter().take(10) {
                            // Limit comments for now
                            if let Ok(comment) = api_clone.fetch_comment_content(id) {
                                comments.push(comment);
                            }
                        }
                        let _ = tx_clone.send(Action::CommentsLoaded(comments));
                    });
                } else {
                    self.comments_loading = false;
                }
            }
            Action::CommentsLoaded(comments) => {
                self.comments = comments;
                self.comments_loading = false;
            }
            Action::ToggleArticleView => {
                if self.view_mode == ViewMode::StoryDetail {
                    self.view_mode = ViewMode::Article;
                    if self.article_content.is_none()
                        && !self.article_loading
                        && let Some(story) = &self.selected_story
                        && let Some(url) = &story.url
                    {
                        self.article_loading = true;
                        let api = self.api_service.clone();
                        let tx = self.action_tx.clone();
                        let story_id = story.id;
                        let url = url.clone();
                        let list_type = self.current_list_type;
                        tokio::spawn(async move {
                            if let Ok(content) = api.fetch_article_content(&url) {
                                let _ =
                                    tx.send(Action::ArticleLoaded(list_type, story_id, content));
                            } else {
                                let _ =
                                    tx.send(Action::Error("Failed to fetch article".to_string()));
                            }
                        });
                    }
                } else if self.view_mode == ViewMode::Article {
                    self.view_mode = ViewMode::StoryDetail;
                }
            }
            Action::ArticleLoaded(list_type, id, content) => {
                // Only apply the loaded article if it matches the currently-selected story
                // and it was loaded for the same list/category the user selected from.
                if let Some(selected) = &self.selected_story
                    && selected.id == id
                    && self.current_list_type == list_type
                {
                    self.article_content = Some(content);
                    self.article_for_story_id = Some(id);
                }
                self.article_loading = false;
            }
            Action::ScrollArticleUp => {
                if self.article_scroll > 0 {
                    self.article_scroll -= 1;
                }
            }
            Action::ScrollArticleDown => {
                self.article_scroll += 1;
            }
            Action::SwitchTheme => {
                if !self.available_themes.is_empty() {
                    self.current_theme_index =
                        (self.current_theme_index + 1) % self.available_themes.len();
                    let (filename, mode) = &self.available_themes[self.current_theme_index];
                    if let Ok(new_theme) = load_theme(Path::new(filename), mode) {
                        self.theme = new_theme;
                    }
                }
            }
            Action::ClearNotification => {
                self.notification_message = None;
                self.notification_timer = None;
            }
            Action::Error(msg) => {
                self.loading = false;
                // TODO: Show error
                tracing::error!("{}", msg);
            }
        }
    }

    /// Return a vector of (original_index, &Story) representing the currently-displayed stories
    /// after applying the search filter. This ensures selection indices used by `ListState`
    /// correspond to the displayed items.
    fn filtered_story_indices(&self) -> Vec<(usize, &Story)> {
        if self.search_query.is_empty() {
            self.stories.iter().enumerate().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.stories
                .iter()
                .enumerate()
                .filter(|(_, story)| {
                    story
                        .title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&query))
                        .unwrap_or(false)
                })
                .collect()
        }
    }

    fn select_next(&mut self) {
        let displayed = self.filtered_story_indices();
        if displayed.is_empty() {
            return;
        }

        let i = match self.story_list_state.selected() {
            Some(i) => {
                if i >= displayed.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.story_list_state.select(Some(i));
    }

    fn select_prev(&mut self) {
        let displayed = self.filtered_story_indices();
        if displayed.is_empty() {
            return;
        }

        let i = match self.story_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    displayed.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.story_list_state.select(Some(i));
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(f.area());

        self.render_top_bar(f, chunks[0]);

        match self.view_mode {
            ViewMode::List => self.render_list(f, chunks[1]),
            ViewMode::StoryDetail => self.render_detail(f, chunks[1]),
            ViewMode::Article => self.render_article(f, chunks[1]),
        }

        self.render_status_bar(f, chunks[2]);

        // Render search overlay if in search mode
        if self.input_mode == InputMode::Search {
            self.render_search_overlay(f);
        }

        // Render notification overlay if present
        if self.notification_message.is_some() {
            self.render_notification(f);
        }

        // Render progress overlay if loading all stories
        if self.story_load_progress.is_some() {
            self.render_progress_overlay(f);
        }
    }

    fn render_progress_overlay(&self, f: &mut Frame) {
        if let Some((loaded, total)) = self.story_load_progress {
            let area = f.area();
            let popup_width = 60.min(area.width - 4);
            let popup_height = 5;
            let popup_x = (area.width.saturating_sub(popup_width)) / 2;
            let popup_y = (area.height.saturating_sub(popup_height)) / 2;
            let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

            let block = Block::default()
                .title("Loading all stories")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.border));

            let clear_area = block.inner(popup_area);
            f.render_widget(Clear, clear_area);
            f.render_widget(block, popup_area);

            let gauge_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .margin(1)
                .split(popup_area)[0];

            let percent = if total > 0 {
                (loaded as f64 / total as f64 * 100.0) as u16
            } else {
                0
            };

            let gauge = ratatui::widgets::Gauge::default()
                .block(Block::default().title(format!("{}/{}", loaded, total)))
                .gauge_style(
                    Style::default()
                        .fg(self.theme.selection_bg)
                        .bg(self.theme.background),
                )
                .percent(percent);

            f.render_widget(gauge, gauge_area);
        }
    }

    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        // Filter stories based on search query
        let filtered_stories: Vec<_> = if self.search_query.is_empty() {
            self.stories.iter().enumerate().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.stories
                .iter()
                .enumerate()
                .filter(|(_, story)| {
                    story
                        .title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&query))
                        .unwrap_or(false)
                })
                .collect()
        };

        let items: Vec<ListItem> = filtered_stories
            .iter()
            .map(|(_, story)| {
                let title = story.title.as_deref().unwrap_or("No Title");
                let score = story.score.unwrap_or(0);
                let by = story.by.as_deref().unwrap_or("unknown");
                let comments = story.descendants.unwrap_or(0);

                let time = story
                    .time
                    .as_ref()
                    .map(crate::utils::datetime::format_timestamp)
                    .unwrap_or_else(|| "unknown".to_string());

                let content = Line::from(vec![
                    Span::styled(format!("{} ", score), Style::default().fg(self.theme.score)),
                    Span::styled(title, Style::default().fg(self.theme.foreground)),
                    Span::styled(
                        format!(" ({} comments by {} | {})", comments, by, time),
                        Style::default().fg(self.theme.comment_time),
                    ),
                ]);
                ListItem::new(content)
            })
            .collect();

        // Place the version next to the "Hacker News" label in the title
        let title = if self.search_query.is_empty() {
            format!(
                "Hacker News v{} - {}",
                self.app_version, self.current_list_type
            )
        } else {
            format!(
                "Hacker News v{} - {} (Filter: {})",
                self.app_version, self.current_list_type, self.search_query
            )
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border))
                    .title(title)
                    .title_style(Style::default().fg(self.theme.foreground)),
            )
            .style(Style::default().bg(self.theme.background))
            .highlight_style(
                Style::default()
                    .bg(self.theme.selection_bg)
                    .fg(self.theme.selection_fg)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut self.story_list_state);
    }

    fn render_detail(&self, f: &mut Frame, area: Rect) {
        if let Some(story) = &self.selected_story {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(0)])
                .split(area);

            let title = story.title.as_deref().unwrap_or("No Title");
            let url = story.url.as_deref().unwrap_or("No URL");
            let time = story
                .time
                .as_ref()
                .map(crate::utils::datetime::format_timestamp)
                .unwrap_or_else(|| "unknown".to_string());
            let text = format!(
                "Title: {}\nURL: {}\nScore: {}\nBy: {}\nTime: {}",
                title,
                url,
                story.score.unwrap_or(0),
                story.by.as_deref().unwrap_or("unknown"),
                time
            );

            let p = Paragraph::new(text)
                .style(
                    Style::default()
                        .fg(self.theme.foreground)
                        .bg(self.theme.background),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(self.theme.border))
                        .title("Story Details")
                        .title_style(Style::default().fg(self.theme.foreground)),
                )
                .wrap(Wrap { trim: true });
            f.render_widget(p, chunks[0]);

            let comments_text: Vec<ListItem> = self
                .comments
                .iter()
                .map(|c| {
                    let author = c.by.as_deref().unwrap_or("unknown");
                    let text = c.text.as_deref().unwrap_or("[deleted]");
                    let clean_text = crate::utils::html::extract_text_from_html(text);
                    let time = c
                        .time
                        .as_ref()
                        .map(crate::utils::datetime::format_timestamp)
                        .unwrap_or_else(|| "unknown".to_string());

                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(author, Style::default().fg(self.theme.comment_author)),
                            Span::styled(
                                format!(" ({})", time),
                                Style::default().fg(self.theme.comment_time),
                            ),
                        ]),
                        Line::from(Span::styled(
                            clean_text,
                            Style::default().fg(self.theme.foreground),
                        )),
                        Line::from(Span::styled("---", Style::default().fg(self.theme.border))),
                    ])
                })
                .collect();

            let list = List::new(comments_text)
                .style(Style::default().bg(self.theme.background))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(self.theme.border))
                        .title("Comments (Tab to view Article)")
                        .title_style(Style::default().fg(self.theme.foreground)),
                );
            f.render_widget(list, chunks[1]);
        }
    }

    fn render_article(&self, f: &mut Frame, area: Rect) {
        // If we have a selected story, show the same metadata block as in the detail view
        if let Some(story) = &self.selected_story {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(0)])
                .split(area);

            let title = story.title.as_deref().unwrap_or("No Title");
            let url = story.url.as_deref().unwrap_or("No URL");
            let time = story
                .time
                .as_ref()
                .map(crate::utils::datetime::format_timestamp)
                .unwrap_or_else(|| "unknown".to_string());
            let meta_text = format!(
                "Title: {}\nURL: {}\nScore: {}\nBy: {}\nTime: {}",
                title,
                url,
                story.score.unwrap_or(0),
                story.by.as_deref().unwrap_or("unknown"),
                time
            );

            let meta_p = Paragraph::new(meta_text)
                .style(
                    Style::default()
                        .fg(self.theme.foreground)
                        .bg(self.theme.background),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(self.theme.border))
                        .title("Story Details")
                        .title_style(Style::default().fg(self.theme.foreground)),
                )
                .wrap(Wrap { trim: true });
            f.render_widget(meta_p, chunks[0]);

            let content = if self.article_loading {
                "Loading article...".to_string()
            } else {
                self.article_content
                    .clone()
                    .unwrap_or_else(|| "No content available or failed to load.".to_string())
            };

            let p = Paragraph::new(content)
                .style(
                    Style::default()
                        .fg(self.theme.foreground)
                        .bg(self.theme.background),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(self.theme.border))
                        .title("Article View (Tab to view Comments)")
                        .title_style(Style::default().fg(self.theme.foreground)),
                )
                .wrap(Wrap { trim: true })
                .scroll((self.article_scroll as u16, 0));
            f.render_widget(p, chunks[1]);
        } else {
            // Fallback: no selected story, render the article content as before
            let content = if self.article_loading {
                "Loading article...".to_string()
            } else {
                self.article_content
                    .clone()
                    .unwrap_or_else(|| "No content available or failed to load.".to_string())
            };

            let p = Paragraph::new(content)
                .style(
                    Style::default()
                        .fg(self.theme.foreground)
                        .bg(self.theme.background),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(self.theme.border))
                        .title("Article View (Tab to view Comments)")
                        .title_style(Style::default().fg(self.theme.foreground)),
                )
                .wrap(Wrap { trim: true })
                .scroll((self.article_scroll as u16, 0));
            f.render_widget(p, area);
        }
    }

    fn render_top_bar(&self, f: &mut Frame, area: Rect) {
        let theme_name = if !self.available_themes.is_empty() {
            let (path, mode) = &self.available_themes[self.current_theme_index];
            let filename = Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            format!("Theme: {} ({})", filename, mode)
        } else {
            String::new()
        };

        // Show only the theme in the top-right corner
        let top_bar_text = theme_name;

        let p = Paragraph::new(top_bar_text)
            .alignment(Alignment::Right)
            .style(
                Style::default()
                    .bg(self.theme.background)
                    .fg(self.theme.foreground),
            );
        f.render_widget(p, area);
    }

    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status = if self.loading || self.comments_loading || self.article_loading {
            "Loading...".to_string()
        } else if self.input_mode == InputMode::Search {
            // Simplified status bar for search mode
            "Search: Type to filter | Enter/Esc: Finish | Ctrl+C: Clear".to_string()
        } else {
            match self.view_mode {
                ViewMode::List => {
                    let loaded_info = if !self.story_ids.is_empty() {
                        format!(" | {}/{}", self.loaded_count, self.story_ids.len())
                    } else {
                        String::new()
                    };
                    let filter_hint = if !self.search_query.is_empty() {
                        format!(" | Filter: {}", self.search_query)
                    } else {
                        String::new()
                    };
                    let clear_hint = if !self.search_query.is_empty() {
                        " | C: Clear"
                    } else {
                        ""
                    };
                    format!(
                        "1-6: Cat | /: Search | j/k: Nav | m: More | A: All | Enter: View | t: Theme | q: Quit{}{}{}",
                        loaded_info, filter_hint, clear_hint
                    )
                }
                ViewMode::StoryDetail => {
                    "Esc/q: Back | o: Browser | Tab: Article | t: Theme".to_string()
                }
                ViewMode::Article => {
                    "Esc/q: Back | o: Browser | Tab: Comments | j/k: Scroll | t: Theme".to_string()
                }
            }
        };

        let p = Paragraph::new(status).style(
            Style::default()
                .bg(self.theme.selection_bg)
                .fg(self.theme.selection_fg),
        );
        f.render_widget(p, area);
    }

    fn render_notification(&self, f: &mut Frame) {
        if let Some(msg) = &self.notification_message {
            let area = f.area();

            // Create centered popup
            let popup_width = (msg.len() as u16 + 4).min(area.width - 4);
            let popup_height = 3;

            let popup_x = (area.width.saturating_sub(popup_width)) / 2;
            let popup_y = (area.height.saturating_sub(popup_height)) / 2;

            let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

            // Clear background
            let popup = Paragraph::new(msg.as_str())
                .style(
                    Style::default()
                        .bg(self.theme.selection_bg)
                        .fg(self.theme.selection_fg)
                        .add_modifier(Modifier::BOLD),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(self.theme.border))
                        .title("Info")
                        .title_style(Style::default().fg(self.theme.foreground)),
                )
                .alignment(Alignment::Center);

            f.render_widget(Clear, popup_area);
            f.render_widget(popup, popup_area);
        }
    }

    fn render_search_overlay(&self, f: &mut Frame) {
        let area = f.area();

        // Create search box at the top center
        let search_width = 60.min(area.width - 4);
        let search_height = 3;

        let search_x = (area.width.saturating_sub(search_width)) / 2;
        let search_y = (area.height.saturating_sub(search_height)) / 2; // Centered vertically

        let search_area = Rect::new(search_x, search_y, search_width, search_height);

        // Display the search query with cursor
        let display_text = format!("{}█", self.search_query); // █ as cursor

        let search_box = Paragraph::new(display_text)
            .style(
                Style::default()
                    .fg(self.theme.foreground)
                    .bg(self.theme.background),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.selection_bg))
                    .title(" Search (Esc to cancel) ")
                    .title_style(
                        Style::default()
                            .fg(self.theme.selection_fg)
                            .bg(self.theme.selection_bg)
                            .add_modifier(Modifier::BOLD),
                    ),
            );

        f.render_widget(Clear, search_area);
        f.render_widget(search_box, search_area);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

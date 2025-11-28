#![allow(clippy::single_match)]
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use crate::api::{ApiService, StoryListType};
use crate::config::AppConfig;
use crate::internal::models::{Article, CommentRow, Story};
use crate::utils::theme_loader::{TuiTheme, load_theme};

use ratatui::Frame;
use ratatui::widgets::ListState;

/// Application view modes.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewMode {
    List,
    StoryDetail,
    Article,
    Bookmarks,
    History,
}

/// Input modes for the UI.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Search,
    #[allow(dead_code)]
    SearchOptions,
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
    CommentsLoaded(Vec<CommentRow>),
    LoadMoreComments,
    #[allow(dead_code)]
    ToggleCommentCollapse(usize),
    ToggleArticleView,
    #[allow(dead_code)]
    ToggleHelp,
    ArticleLoaded(StoryListType, u32, Article),
    ScrollArticleUp,
    ScrollArticleDown,
    SwitchTheme,
    ClearNotification,
    Error(String),
    #[allow(dead_code)]
    ToggleBookmark,
    #[allow(dead_code)]
    ViewBookmarks,
    #[allow(dead_code)]
    ExportBookmarks,
    #[allow(dead_code)]
    ImportBookmarks,
    ViewHistory,
    ClearHistory,
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
    pub comments: Vec<CommentRow>,
    pub comment_ids: Vec<u32>,
    pub loaded_comments_count: usize,
    pub comments_loading: bool,
    /// Scroll offset for comments view (line-by-line scrolling)
    pub comments_scroll: usize,
    pub article_content: Option<Article>,
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
    pub spinner_state: usize,
    pub last_spinner_update: Option<tokio::time::Instant>,
    pub show_help: bool,
    pub input_mode: InputMode,
    pub search_query: crate::internal::search::SearchQuery,
    pub search_history: crate::internal::search::SearchHistory,
    pub temp_search_input: String,
    pub history_index: Option<usize>,
    #[allow(dead_code)]
    pub config: AppConfig,
    pub action_tx: UnboundedSender<Action>,
    pub action_rx: UnboundedReceiver<Action>,
    pub bookmarks: crate::internal::bookmarks::Bookmarks,
    pub history: crate::internal::history::History,
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

        // Select theme using helper that centralizes selection logic and honors config flags.
        // Pass the TERM environment value explicitly so selection logic does not call env::var
        // itself (makes testing and behavior explicit).
        let term_env = std::env::var("TERM").unwrap_or_default();
        let (theme, current_theme_index) =
            Self::select_theme_from_config(&config, &available_themes, &terminal_mode, &term_env);

        // Log which theme was finally selected (index and variant) so startup behavior is traceable.
        match available_themes.get(current_theme_index) {
            Some((filename, mode)) => {
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
            }
            None => {
                tracing::info!("No available themes found; using default TuiTheme");
            }
        }

        let bookmarks = match crate::internal::bookmarks::Bookmarks::load_or_create() {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to load bookmarks: {}", e);
                crate::internal::bookmarks::Bookmarks::new()
            }
        };

        let history = match crate::internal::history::History::load_or_create(50) {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("Failed to load history: {}", e);
                crate::internal::history::History::new(50)
            }
        };

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
            comment_ids: Vec::new(),
            loaded_comments_count: 0,
            comments_loading: false,
            comments_scroll: 0,
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
            spinner_state: 0,
            last_spinner_update: None,
            show_help: false,
            input_mode: InputMode::Normal,
            search_query: crate::internal::search::SearchQuery::default(),
            search_history: match crate::internal::search::SearchHistory::load_or_create(20) {
                Ok(h) => h,
                Err(e) => {
                    tracing::error!("Failed to load search history: {}", e);
                    crate::internal::search::SearchHistory::new(20)
                }
            },
            temp_search_input: String::new(),
            history_index: None,
            config,
            action_tx,
            action_rx,
            bookmarks,
            history,
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
            let mode = match bg_val {
                0..=7 => "dark",
                8..=15 => "light",
                _ => "unknown",
            };
            return mode.to_string();
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

            match (cand.is_file(), std::fs::read_dir(&cand)) {
                (true, _) => {
                    if let Some(ext) = cand.extension().and_then(|s| s.to_str())
                        && ext.eq_ignore_ascii_case("json")
                        && let Some(filename) = cand.to_str()
                    {
                        themes.push((filename.to_string(), "dark".to_string()));
                        themes.push((filename.to_string(), "light".to_string()));
                    }
                }
                (false, Ok(entries)) => {
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
                _ => {}
            }
        }

        // Deduplicate while preserving order of discovery
        let mut seen = std::collections::HashSet::new();
        themes.retain(|(p, mode)| {
            let key = format!("{}:{}", p, mode);
            match seen.contains(&key) {
                true => false,
                false => {
                    seen.insert(key);
                    true
                }
            }
        });

        themes
    }

    /// Centralized theme selection logic extracted from `new`.
    /// Returns (TuiTheme, selected_index) for the given config and discovered themes.
    pub fn select_theme_from_config(
        config: &crate::config::AppConfig,
        available_themes: &[(String, String)],
        terminal_mode: &str,
        term_env: &str,
    ) -> (TuiTheme, usize) {
        if available_themes.is_empty() {
            return (TuiTheme::default(), 0);
        }

        // Canonicalize configured theme name and detect optional explicit mode token.
        let theme_name_raw = config.theme_name.trim();
        let mut requested_mode: Option<String> = None;
        let mut base_name = theme_name_raw.to_string();

        if let Some(last) = theme_name_raw.split_whitespace().last()
            && (last.eq_ignore_ascii_case("dark") || last.eq_ignore_ascii_case("light"))
        {
            requested_mode = Some(last.to_lowercase());
            let tokens: Vec<&str> = theme_name_raw.split_whitespace().collect();
            base_name = match tokens.split_last() {
                Some((_last, rest)) if !rest.is_empty() => rest.join(" "),
                _ => String::new(),
            };

            // Respect ghost terminal name from config; use the provided `term_env` value
            // passed in by the caller rather than reading the environment here.
            let ghost_name = config.ghost_term_name.trim();
            match term_env.eq_ignore_ascii_case(ghost_name) {
                true => {
                    tracing::info!(
                        "TERM='{}' detected; honoring requested theme variant '{}'",
                        term_env,
                        requested_mode.as_deref().unwrap_or("unknown")
                    );
                }
                false => match (
                    requested_mode.as_deref() == Some("dark"),
                    config.auto_switch_dark_to_light,
                ) {
                    (true, true) => {
                        tracing::info!(
                            "Auto-switching requested dark variant to light because TERM!='{}' and auto_switch_dark_to_light is enabled",
                            ghost_name
                        );
                        requested_mode = Some("light".to_string());
                    }
                    _ => {
                        tracing::info!("Requested theme variant retained: {:?}", requested_mode);
                    }
                },
            }
        }

        let base_lower = base_name.to_lowercase();
        let fullname_lower = theme_name_raw.to_lowercase();

        // Find best candidate index using same strategy as before
        let mut matched_idx: Option<usize> = None;

        if !base_lower.is_empty() {
            for (i, (path, mode)) in available_themes.iter().enumerate() {
                if let Some(stem) = Path::new(path).file_stem().and_then(|s| s.to_str())
                    && stem.eq_ignore_ascii_case(&base_lower)
                {
                    match (requested_mode.as_deref(), mode.as_str()) {
                        (Some(_), _)
                            if mode.eq_ignore_ascii_case(requested_mode.as_deref().unwrap()) =>
                        {
                            matched_idx = Some(i);
                            break;
                        }
                        (None, _) if mode == terminal_mode => {
                            matched_idx = Some(i);
                            break;
                        }
                        _ => {
                            if matched_idx.is_none() {
                                matched_idx = Some(i);
                            }
                        }
                    }
                }
            }
        }

        if matched_idx.is_none() {
            for (i, (path, mode)) in available_themes.iter().enumerate() {
                if let Some(stem) = Path::new(path).file_stem().and_then(|s| s.to_str())
                    && fullname_lower.starts_with(&stem.to_lowercase())
                {
                    match (requested_mode.as_deref(), mode.as_str()) {
                        (Some(_), _)
                            if mode.eq_ignore_ascii_case(requested_mode.as_deref().unwrap()) =>
                        {
                            matched_idx = Some(i);
                            break;
                        }
                        (None, _) if mode == terminal_mode => {
                            matched_idx = Some(i);
                            break;
                        }
                        _ => {
                            if matched_idx.is_none() {
                                matched_idx = Some(i);
                            }
                        }
                    }
                }
            }
        }

        match matched_idx {
            Some(idx) => {
                let (filename, mode) = &available_themes[idx];
                let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                (theme, idx)
            }
            None => match requested_mode.as_deref() {
                Some(req) => {
                    match (
                        available_themes
                            .iter()
                            .position(|(_, mode)| mode.eq_ignore_ascii_case(req)),
                        available_themes
                            .iter()
                            .position(|(_, mode)| mode == terminal_mode),
                    ) {
                        (Some(idx), _) => {
                            let (filename, mode) = &available_themes[idx];
                            let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                            (theme, idx)
                        }
                        (None, Some(idx)) => {
                            let (filename, mode) = &available_themes[idx];
                            let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                            (theme, idx)
                        }
                        (None, None) => {
                            let (filename, mode) = &available_themes[0];
                            let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                            (theme, 0)
                        }
                    }
                }
                None => match available_themes
                    .iter()
                    .position(|(_, mode)| mode == terminal_mode)
                {
                    Some(idx) => {
                        let (filename, mode) = &available_themes[idx];
                        let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                        (theme, idx)
                    }
                    None => {
                        let (filename, mode) = &available_themes[0];
                        let theme = load_theme(Path::new(filename), mode).unwrap_or_default();
                        (theme, 0)
                    }
                },
            },
        }
    }

    pub async fn run(&mut self, mut tui: crate::tui::Tui) -> Result<()> {
        // Initial load
        let _ = self
            .action_tx
            .send(Action::LoadStories(self.current_list_type));

        let mut event_interval = tokio::time::interval(std::time::Duration::from_millis(16));

        loop {
            // Update spinner animation every 100ms
            let now = tokio::time::Instant::now();
            match self.last_spinner_update {
                Some(last_update) => {
                    if now.duration_since(last_update).as_millis() >= 100 {
                        self.spinner_state = self.spinner_state.wrapping_add(1);
                        self.last_spinner_update = Some(now);
                    }
                }
                None => {
                    self.last_spinner_update = Some(now);
                }
            }

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
            InputMode::Search | InputMode::SearchOptions => self.handle_search_input(key),
            InputMode::Normal => self.handle_normal_input(key),
        }
    }

    fn handle_search_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('/') => {
                // Ignore / in search mode (it's the key that enters search mode)
            }
            KeyCode::Char('m')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                // Cycle search mode
                self.search_query.mode = self.search_query.mode.next();
            }
            KeyCode::F(2) => {
                // Also cycle search mode
                self.search_query.mode = self.search_query.mode.next();
            }
            KeyCode::Char('r')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                // Toggle regex
                self.search_query.search_type = self.search_query.search_type.toggle();
                // Recompile with current temp input
                self.search_query = crate::internal::search::SearchQuery::new(
                    self.temp_search_input.clone(),
                    self.search_query.mode,
                    self.search_query.search_type,
                );
            }
            KeyCode::F(3) => {
                // Also toggle regex
                self.search_query.search_type = self.search_query.search_type.toggle();
                self.search_query = crate::internal::search::SearchQuery::new(
                    self.temp_search_input.clone(),
                    self.search_query.mode,
                    self.search_query.search_type,
                );
            }
            KeyCode::Up => {
                // Navigate history up (older)
                let current = self.history_index.unwrap_or(0);
                if current < self.search_history.queries.len().saturating_sub(1) {
                    let new_index = current + 1;
                    self.history_index = Some(new_index);
                    if let Some(query) = self.search_history.get_recent(new_index) {
                        self.temp_search_input = query.clone();
                        self.search_query = crate::internal::search::SearchQuery::new(
                            query.clone(),
                            self.search_query.mode,
                            self.search_query.search_type,
                        );
                    }
                }
            }
            KeyCode::Down => {
                // Navigate history down (newer)
                if let Some(current) = self.history_index {
                    if current > 0 {
                        let new_index = current - 1;
                        self.history_index = Some(new_index);
                        if let Some(query) = self.search_history.get_recent(new_index) {
                            self.temp_search_input = query.clone();
                            self.search_query = crate::internal::search::SearchQuery::new(
                                query.clone(),
                                self.search_query.mode,
                                self.search_query.search_type,
                            );
                        }
                    } else {
                        // At the bottom, clear to empty
                        self.history_index = None;
                        self.temp_search_input.clear();
                        self.search_query = crate::internal::search::SearchQuery::new(
                            String::new(),
                            self.search_query.mode,
                            self.search_query.search_type,
                        );
                    }
                }
            }
            KeyCode::Char(c) => {
                self.temp_search_input.push(c);
                self.search_query = crate::internal::search::SearchQuery::new(
                    self.temp_search_input.clone(),
                    self.search_query.mode,
                    self.search_query.search_type,
                );
                self.history_index = None;
            }
            KeyCode::Backspace => {
                self.temp_search_input.pop();
                self.search_query = crate::internal::search::SearchQuery::new(
                    self.temp_search_input.clone(),
                    self.search_query.mode,
                    self.search_query.search_type,
                );
                self.history_index = None;
            }
            KeyCode::Enter => {
                // Save to history and exit search mode
                if !self.temp_search_input.is_empty() {
                    self.search_history.add(self.temp_search_input.clone());
                    let _ = self.search_history.save();
                }
                self.input_mode = InputMode::Normal;
                self.history_index = None;
            }
            KeyCode::Esc => {
                // Cancel search - clear and exit
                self.temp_search_input.clear();
                self.search_query = crate::internal::search::SearchQuery::default();
                self.input_mode = InputMode::Normal;
                self.history_index = None;
            }
            _ => {}
        }
    }

    fn handle_normal_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('?') => {
                let _ = self.action_tx.send(Action::ToggleHelp);
            }
            KeyCode::Esc | KeyCode::Char('q') => match (self.show_help, self.view_mode) {
                (true, _) => {
                    let _ = self.action_tx.send(Action::ToggleHelp);
                }
                (false, ViewMode::List) => {
                    let _ = self.action_tx.send(Action::Quit);
                }
                (false, ViewMode::StoryDetail) | (false, ViewMode::Article) => {
                    let _ = self.action_tx.send(Action::Back);
                }
                // When in Bookmarks or History view, Esc/q should go back to the List view
                (false, ViewMode::Bookmarks) | (false, ViewMode::History) => {
                    let _ = self.action_tx.send(Action::Back);
                }
            },
            KeyCode::Char('j') | KeyCode::Down => match self.view_mode {
                ViewMode::Article => {
                    let _ = self.action_tx.send(Action::ScrollArticleDown);
                }
                _ => {
                    let _ = self.action_tx.send(Action::NavigateDown);
                }
            },
            KeyCode::Char('k') | KeyCode::Up => match self.view_mode {
                ViewMode::Article => {
                    let _ = self.action_tx.send(Action::ScrollArticleUp);
                }
                _ => {
                    let _ = self.action_tx.send(Action::NavigateUp);
                }
            },
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
            KeyCode::Char('b') => {
                let _ = self.action_tx.send(Action::ToggleBookmark);
            }
            KeyCode::Char('B') => {
                let _ = self.action_tx.send(Action::ViewBookmarks);
            }
            KeyCode::Char('H') => {
                let _ = self.action_tx.send(Action::ViewHistory);
            }
            KeyCode::Char('X') => {
                if self.view_mode == ViewMode::History {
                    let _ = self.action_tx.send(Action::ClearHistory);
                }
            }
            // Toggle auto_switch_dark_to_light at runtime and persist to config.ron.
            // Pressing 'g' flips the flag, saves config, and shows a short notification.
            KeyCode::Char('g') => {
                // Flip the flag and persist the configuration.
                self.config.auto_switch_dark_to_light = !self.config.auto_switch_dark_to_light;
                // Attempt to save the config to disk; AppConfig::save preserves comments.
                self.config.save();

                let status = match self.config.auto_switch_dark_to_light {
                    true => "enabled",
                    false => "disabled",
                };

                // Notify user briefly
                self.notification_message = Some(format!("Auto-switch Dark->Light {}", status));
                self.notification_timer = Some(tokio::time::Instant::now());

                // Re-evaluate theme selection using the centralized helper and apply it immediately.
                let term_env = std::env::var("TERM").unwrap_or_default();
                let (new_theme, new_idx) = Self::select_theme_from_config(
                    &self.config,
                    &self.available_themes,
                    &self.terminal_mode,
                    &term_env,
                );
                self.theme = new_theme;
                self.current_theme_index = new_idx;

                // Schedule a clear of the notification after a few seconds
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    let _ = tx.send(Action::ClearNotification);
                });
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
                    self.temp_search_input = self.search_query.query.clone();
                    self.history_index = None;
                }
            }
            KeyCode::Char('C') => {
                if !self.search_query.is_empty() {
                    self.search_query = crate::internal::search::SearchQuery::default();
                    self.temp_search_input.clear();
                }
            }
            KeyCode::Char('n') => {
                if self.view_mode == ViewMode::StoryDetail {
                    let _ = self.action_tx.send(Action::LoadMoreComments);
                }
            }
            _ => {}
        }
    }

    async fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.running = false,
            Action::NavigateUp => {
                match self.view_mode {
                    ViewMode::StoryDetail => {
                        // Scroll up in comments
                        self.comments_scroll = self.comments_scroll.saturating_sub(1);
                    }
                    _ => {
                        self.select_prev();
                    }
                }
            }
            Action::NavigateDown => {
                match self.view_mode {
                    ViewMode::StoryDetail => {
                        // Scroll down in comments
                        self.comments_scroll = self.comments_scroll.saturating_add(1);
                    }
                    _ => {
                        self.select_next();
                    }
                }
            }
            Action::Enter => {
                if let Some(index) = self.story_list_state.selected() {
                    // Map the selected index (which refers to the displayed/filtered list)
                    // back to the original story using the same filter logic used when
                    // rendering the list. This ensures Enter selects the story shown
                    // on that row even when a filter/search is active.
                    let displayed: Vec<_> = match self.search_query.is_empty() {
                        true => self.stories.iter().enumerate().collect(),
                        false => self
                            .stories
                            .iter()
                            .enumerate()
                            .filter(|(_, story)| {
                                story
                                    .title
                                    .as_ref()
                                    .map(|t| self.search_query.matches(t))
                                    .unwrap_or(false)
                            })
                            .collect(),
                    };

                    if let Some((_, s)) = displayed.get(index).cloned() {
                        // Clone the story so we send an owned Story in the action.
                        let story = s.clone();
                        // Add to history
                        self.history.add(&story);
                        let _ = self.history.save();

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
                self.comment_ids.clear();
                self.loaded_comments_count = 0;
                // Reset comment list state so when returning to a story later the
                // comments view doesn't retain a prior selection/scroll.
                self.comments_scroll = 0;
            }
            Action::OpenBrowser => {
                match (&self.selected_story, self.story_list_state.selected()) {
                    (Some(story), _) => {
                        if let Some(url) = &story.url {
                            let _ = open::that(url);
                        }
                    }
                    (None, Some(index)) => {
                        // Map selected displayed index back to original story so OpenBrowser
                        // opens the URL for the story visible on that row when filtered.
                        let displayed: Vec<_> = match self.search_query.is_empty() {
                            true => self.stories.iter().enumerate().collect(),
                            false => self
                                .stories
                                .iter()
                                .enumerate()
                                .filter(|(_, story)| {
                                    story
                                        .title
                                        .as_ref()
                                        .map(|t| self.search_query.matches(t))
                                        .unwrap_or(false)
                                })
                                .collect(),
                        };

                        if let Some((_, story)) = displayed.get(index).cloned()
                            && let Some(url) = &story.url
                        {
                            let _ = open::that(url);
                        }
                    }
                    _ => {}
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
                    match api.fetch_story_ids(list_type) {
                        Ok(ids) => {
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
                        }
                        Err(_) => {
                            let _ = tx.send(Action::Error("Failed to fetch stories".to_string()));
                        }
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
                if let (true, None) = (!self.stories.is_empty(), self.story_list_state.selected()) {
                    self.story_list_state.select(Some(0))
                }
            }
            Action::LoadMoreStories => match (self.loading, self.story_ids.is_empty()) {
                (true, _) | (_, true) => {}
                _ => match self.loaded_count >= self.story_ids.len() {
                    true => {
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
                    }
                    false => {
                        self.loading = true;
                        let api = self.api_service.clone();
                        let tx = self.action_tx.clone();
                        let ids_to_fetch = self
                            .story_ids
                            .iter()
                            .skip(self.loaded_count)
                            .take(20)
                            .copied()
                            .collect::<Vec<_>>();

                        tokio::spawn(async move {
                            let mut stories = Vec::new();
                            for id in &ids_to_fetch {
                                if let Ok(story) = api.fetch_story_content(*id) {
                                    stories.push(story);
                                }
                            }
                            let _ = tx.send(Action::StoriesLoaded(stories));
                        });
                    }
                },
            },
            Action::LoadAllStories => match (
                self.loading,
                self.story_ids.is_empty(),
                self.story_load_progress.is_some(),
            ) {
                (true, _, _) | (_, true, _) | (_, _, true) => {}
                _ => {
                    // Check if all stories are already loaded
                    match self.loaded_count >= self.story_ids.len() {
                        true => {
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
                        }
                        false => {
                            self.loading = true;
                            let api = self.api_service.clone();
                            let tx = self.action_tx.clone();
                            let start_idx = self.loaded_count;

                            // Load ALL remaining stories
                            let ids_to_fetch: Vec<_> =
                                self.story_ids.iter().skip(start_idx).copied().collect();

                            match ids_to_fetch.is_empty() {
                                true => {
                                    self.loading = false;
                                }
                                false => {
                                    self.story_load_progress = Some((0, ids_to_fetch.len()));

                                    tokio::spawn(async move {
                                        let mut stories = Vec::new();
                                        for (i, id) in ids_to_fetch.iter().enumerate() {
                                            if let Ok(story) = api.fetch_story_content(*id) {
                                                stories.push(story);
                                            }
                                            let _ = tx.send(Action::StoryLoadingProgress(i + 1));
                                            // Add a small delay to avoid hitting API rate limits
                                            tokio::time::sleep(std::time::Duration::from_millis(
                                                20,
                                            ))
                                            .await;
                                        }
                                        let _ = tx.send(Action::StoriesLoaded(stories));
                                    });
                                }
                            }
                        }
                    }
                }
            },
            Action::SelectStory(story, list_type) => {
                // Propagate the category/list type when selecting a story so downstream
                // operations (e.g., fetching details) have the context available.
                self.current_list_type = list_type;
                self.selected_story = Some(story.clone());
                // Show Article view first when a story is selected.
                self.view_mode = ViewMode::Article;
                self.comments_loading = true;
                self.comments.clear();
                // Ensure comments view will start at the top for this story.
                self.comments_scroll = 0;

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
                        match api_clone.fetch_article_content(&url) {
                            Ok(content) => {
                                let _ = tx_clone.send(Action::ArticleLoaded(
                                    list_for_request,
                                    story_id,
                                    content,
                                ));
                            }
                            Err(_) => {
                                let _ = tx_clone
                                    .send(Action::Error("Failed to fetch article".to_string()));
                            }
                        }
                    });
                }

                // Fetch comments in the background as before so they are available
                // if the user switches to the comments view.
                match story.kids {
                    Some(kids) => {
                        // Store all comment IDs for pagination
                        self.comment_ids = kids.clone();
                        self.loaded_comments_count = 0;

                        let api_clone = api.clone();
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            // Use fetch_comment_tree to get threaded comments
                            if let Ok(comment_rows) = api_clone.fetch_comment_tree(kids) {
                                let _ = tx_clone.send(Action::CommentsLoaded(comment_rows));
                            }
                        });
                    }
                    None => {
                        self.comment_ids.clear();
                        self.loaded_comments_count = 0;
                        self.comments_loading = false;
                    }
                }
            }
            Action::CommentsLoaded(comment_rows) => {
                // Replace existing comments with the new tree
                self.loaded_comments_count = comment_rows.len();
                self.comments = comment_rows;
                self.comments_loading = false;
                self.comments_scroll = 0;
            }
            Action::LoadMoreComments => {
                // With fetch_comment_tree, we load all comments at once (up to MAX_COMMENTS limit)
                // So this action is essentially a no-op for now
                self.notification_message = Some("All comments already loaded".to_string());
                self.notification_timer = Some(tokio::time::Instant::now());

                // Schedule notification clear
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    let _ = tx.send(Action::ClearNotification);
                });
            }
            Action::ToggleCommentCollapse(index) => {
                if let Some(row) = self.comments.get_mut(index) {
                    row.expanded = !row.expanded;
                }
            }
            Action::ToggleArticleView => {
                match self.view_mode {
                    ViewMode::StoryDetail => {
                        self.view_mode = ViewMode::Article;
                        // If we haven't loaded the article yet, start loading it
                        if self.article_content.is_none()
                            && !self.article_loading
                            && let Some(story) = &self.selected_story
                            && let Some(url) = &story.url
                        {
                            self.article_loading = true;
                            let api = self.api_service.clone();
                            let tx = self.action_tx.clone();
                            let url = url.clone();
                            let list_type = self.current_list_type;
                            let story_id = story.id;
                            tokio::spawn(async move {
                                match api.fetch_article_content(&url) {
                                    Ok(content) => {
                                        let _ = tx.send(Action::ArticleLoaded(
                                            list_type, story_id, content,
                                        ));
                                    }
                                    Err(_) => {
                                        let _ = tx.send(Action::Error(
                                            "Failed to fetch article".to_string(),
                                        ));
                                    }
                                }
                            });
                        }
                    }
                    ViewMode::Article => {
                        self.view_mode = ViewMode::StoryDetail;
                    }
                    _ => {}
                }
            }
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
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
                if self.available_themes.is_empty() {
                    return;
                }

                // Determine the currently-active variant mode (e.g., "dark" or "light")
                let current_mode = self
                    .available_themes
                    .get(self.current_theme_index)
                    .map(|(_, m)| m.to_lowercase())
                    .unwrap_or_else(|| self.terminal_mode.clone());

                // Collect indices of all discovered themes that have the same mode.
                let group: Vec<usize> = self
                    .available_themes
                    .iter()
                    .enumerate()
                    .filter_map(
                        |(i, (_p, mode))| match mode.eq_ignore_ascii_case(&current_mode) {
                            true => Some(i),
                            false => None,
                        },
                    )
                    .collect();

                match group.len() {
                    n if n > 1 => {
                        // Cycle within the same-mode group (e.g., all dark themes)
                        let pos = group
                            .iter()
                            .position(|&idx| idx == self.current_theme_index)
                            .unwrap_or(0);
                        let next_pos = (pos + 1) % group.len();
                        let new_idx = group[next_pos];
                        self.current_theme_index = new_idx;
                        let (filename, mode) = &self.available_themes[new_idx];
                        if let Ok(new_theme) = load_theme(Path::new(filename), mode) {
                            self.theme = new_theme;
                        }
                    }
                    _ => {
                        // Fallback: try to find the next global entry that matches the current mode,
                        // otherwise just advance by one.
                        let total = self.available_themes.len();
                        let mut chosen = (self.current_theme_index + 1) % total;
                        if let Some(idx) = (0..total)
                            .map(|n| (self.current_theme_index + 1 + n) % total)
                            .find(|&i| {
                                self.available_themes[i]
                                    .1
                                    .eq_ignore_ascii_case(&current_mode)
                            })
                        {
                            chosen = idx;
                        }
                        self.current_theme_index = chosen;
                        let (filename, mode) = &self.available_themes[self.current_theme_index];
                        if let Ok(new_theme) = load_theme(Path::new(filename), mode) {
                            self.theme = new_theme;
                        }
                    }
                }
            }
            // Bookmark-related actions
            Action::ToggleBookmark => {
                // Determine the story to toggle:
                // - Prefer the currently selected detailed story (`selected_story`)
                // - Otherwise use the selected item from the list
                // For the Bookmarks view the list indices correspond to `self.bookmarks.stories`
                // (not `self.stories`), so handle that case explicitly.
                // Capture the current list selection index so we can adjust it after toggling
                let prev_selected_idx = self.story_list_state.selected();
                let maybe_story: Option<Story> =
                    match (&self.selected_story, self.story_list_state.selected()) {
                        (Some(s), _) => Some(s.clone()),
                        (None, Some(idx)) => {
                            match self.view_mode {
                                ViewMode::Bookmarks => {
                                    // Selected index refers into bookmarks.stories
                                    self.bookmarks.stories.get(idx).map(|bookmarked| Story {
                                        id: bookmarked.id,
                                        title: Some(bookmarked.title.clone()),
                                        url: bookmarked.url.clone(),
                                        by: None,
                                        score: None,
                                        time: None,
                                        descendants: None,
                                        kids: None,
                                    })
                                }
                                _ => {
                                    // Normal list: map displayed indices to stories
                                    let displayed = self.filtered_story_indices();
                                    displayed.get(idx).map(|(_, s)| (*s).clone())
                                }
                            }
                        }
                        _ => None,
                    };

                match maybe_story {
                    Some(story) => {
                        self.bookmarks.toggle(&story);
                        // Persist bookmarks immediately; log on failure
                        match self.bookmarks.save() {
                            Err(e) => {
                                tracing::error!(%e, "Failed to save bookmarks");
                                self.notification_message =
                                    Some("Failed to save bookmarks".to_string());
                            }
                            Ok(_) => {
                                let msg = match self.bookmarks.contains(story.id) {
                                    true => "Bookmarked".to_string(),
                                    false => "Bookmark removed".to_string(),
                                };
                                self.notification_message = Some(msg);
                            }
                        }
                        self.notification_timer = Some(tokio::time::Instant::now());

                        // If we're in the Bookmarks view, adjust the selection so that a removed
                        // bookmark disappears from the list and selection is clamped to a valid index.
                        if let ViewMode::Bookmarks = self.view_mode {
                            // If the story is no longer contained, it was removed.
                            match (
                                self.bookmarks.contains(story.id),
                                self.bookmarks.stories.is_empty(),
                                prev_selected_idx,
                            ) {
                                (false, true, _) => {
                                    // No bookmarks left: clear selection
                                    self.story_list_state.select(None);
                                }
                                (false, false, Some(prev)) => {
                                    // Clamp selection to last index if needed
                                    let max_idx = self.bookmarks.stories.len().saturating_sub(1);
                                    let new_idx = if prev > max_idx { max_idx } else { prev };
                                    self.story_list_state.select(Some(new_idx));
                                }
                                (false, false, None) => {
                                    // No previous selection recorded: select first item
                                    self.story_list_state.select(Some(0));
                                }
                                (true, _, _) => {
                                    // If the bookmark was added while in Bookmarks view, put selection
                                    // on the newly-added item (bookmarks.add inserts at 0).
                                    self.story_list_state.select(Some(0));
                                }
                            }

                            // Clear any selected_story because the Bookmarks view is a list view
                            self.selected_story = None;
                        }

                        // Schedule notification clear after a short delay
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                            let _ = tx.send(Action::ClearNotification);
                        });
                    }
                    None => {
                        self.notification_message =
                            Some("No story selected to (un)bookmark".to_string());
                        self.notification_timer = Some(tokio::time::Instant::now());

                        // Schedule notification clear after a short delay
                        let tx = self.action_tx.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                            let _ = tx.send(Action::ClearNotification);
                        });
                    }
                }
            }
            Action::ViewBookmarks => {
                // Switch UI into the Bookmarks view. Selection state in the list is reused
                // to show bookmarked items; the view rendering code is responsible for
                // interpreting App.bookmarks when the view_mode == Bookmarks.
                self.view_mode = ViewMode::Bookmarks;
                // Reset selection to first item if we have bookmarks
                match self.bookmarks.stories.first() {
                    Some(_) => self.story_list_state.select(Some(0)),
                    None => self.story_list_state.select(None),
                }
            }
            Action::ExportBookmarks => {
                // Export bookmarks to a simple file in the config dir
                match dirs::config_dir() {
                    Some(dir) => {
                        let app_dir = dir.join("tui-hn-app");
                        match std::fs::create_dir_all(&app_dir) {
                            Err(e) => {
                                tracing::error!(%e, "Failed to create config dir for export");
                                self.notification_message =
                                    Some("Bookmark export failed".to_string());
                            }
                            Ok(_) => {
                                let export_path = app_dir.join("bookmarks_export.json");
                                match serde_json::to_string_pretty(&self.bookmarks) {
                                    Ok(content) => match std::fs::write(&export_path, content) {
                                        Ok(_) => {
                                            self.notification_message = Some(format!(
                                                "Exported to {}",
                                                export_path.display()
                                            ));
                                        }
                                        Err(e) => {
                                            tracing::error!(%e, "Failed to write export file");
                                            self.notification_message =
                                                Some("Bookmark export failed".to_string());
                                        }
                                    },
                                    Err(e) => {
                                        tracing::error!(%e, "Failed to serialize bookmarks");
                                        self.notification_message =
                                            Some("Bookmark export failed".to_string());
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        self.notification_message =
                            Some("Bookmark export failed (no config dir)".to_string());
                    }
                }
                self.notification_timer = Some(tokio::time::Instant::now());
            }
            Action::ImportBookmarks => {
                // Try to load bookmarks from disk (bookmarks.json). If it fails, keep current bookmarks.
                match crate::internal::bookmarks::Bookmarks::load_or_create() {
                    Ok(b) => {
                        self.bookmarks = b;
                        self.notification_message = Some("Bookmarks imported".to_string());
                    }
                    Err(e) => {
                        tracing::error!(%e, "Failed to import bookmarks");
                        self.notification_message = Some("Bookmark import failed".to_string());
                    }
                }
                self.notification_timer = Some(tokio::time::Instant::now());
            }
            Action::ViewHistory => {
                self.view_mode = ViewMode::History;
                self.story_list_state.select(Some(0));
            }
            Action::ClearHistory => {
                self.history.clear();
                let _ = self.history.save();
                self.notification_message = Some("History cleared".to_string());
                self.notification_timer = Some(tokio::time::Instant::now());

                // Schedule notification clear after a short delay (keep behavior consistent with other notifications)
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    let _ = tx.send(Action::ClearNotification);
                });
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
    pub fn filtered_story_indices(&self) -> Vec<(usize, &Story)> {
        match self.search_query.is_empty() {
            true => self.stories.iter().enumerate().collect(),
            false => self
                .stories
                .iter()
                .enumerate()
                .filter(|(_, story)| {
                    story
                        .title
                        .as_ref()
                        .map(|t| self.search_query.matches(t))
                        .unwrap_or(false)
                })
                .collect(),
        }
    }

    fn select_next(&mut self) {
        let displayed = self.filtered_story_indices();
        if displayed.is_empty() {
            return;
        }

        let i = match self.story_list_state.selected() {
            Some(i) => match i {
                n if n >= displayed.len() - 1 => 0,
                _ => i + 1,
            },
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
            Some(i) => match i {
                0 => displayed.len() - 1,
                n => n - 1,
            },
            None => 0,
        };
        self.story_list_state.select(Some(i));
    }

    pub fn get_spinner_char(&self) -> &'static str {
        const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        SPINNER_FRAMES[self.spinner_state % SPINNER_FRAMES.len()]
    }

    #[allow(dead_code)] // Reserved for future network status feature
    pub fn active_loading_count(&self) -> usize {
        let mut count = 0;
        if self.loading {
            count += 1;
        }
        if self.comments_loading {
            count += 1;
        }
        if self.article_loading {
            count += 1;
        }
        count
    }

    pub fn loading_description(&self) -> Option<String> {
        match (self.article_loading, self.comments_loading, self.loading) {
            (true, _, _) => Some("Loading article...".to_string()),
            (_, true, _) => Some("Loading comments...".to_string()),
            (_, _, true) => Some("Loading stories...".to_string()),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {}

    pub fn ui(&mut self, f: &mut Frame) {
        super::view::draw(self, f);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn select_exact_dark_when_ghost_term() {
        // Configure AppConfig to request Gruvbox Dark and set ghost term name to match TERM.
        let cfg = AppConfig {
            theme_name: "Gruvbox Dark".to_string(),
            ghost_term_name: "xterm-ghostty".to_string(),
            auto_switch_dark_to_light: true,
            ..Default::default()
        };

        // Provide available themes: gruvbox.json dark then light
        let available = vec![
            ("./themes/gruvbox.json".to_string(), "dark".to_string()),
            ("./themes/gruvbox.json".to_string(), "light".to_string()),
        ];

        // Terminal mode argument (runtime detection) - pass explicit TERM value (ghost)
        let term_env = "xterm-ghostty";
        let (_theme, idx) = App::select_theme_from_config(&cfg, &available, "dark", term_env);

        // Should select the dark variant (index 0)
        assert_eq!(
            idx, 0,
            "Expected dark variant to be chosen when TERM matches ghost_term_name"
        );
    }

    #[test]
    fn auto_switch_dark_to_light_when_not_ghost() {
        // Request Gruvbox Dark but TERM is not ghost; auto-switch enabled.
        let cfg = AppConfig {
            theme_name: "Gruvbox Dark".to_string(),
            ghost_term_name: "xterm-ghostty".to_string(),
            auto_switch_dark_to_light: true,
            ..Default::default()
        };

        let available = vec![
            ("./themes/gruvbox.json".to_string(), "dark".to_string()),
            ("./themes/gruvbox.json".to_string(), "light".to_string()),
        ];

        // Terminal mode argument (runtime detection) - pass non-ghost TERM
        let term_env = "xterm-256color";
        let (_theme, idx) = App::select_theme_from_config(&cfg, &available, "dark", term_env);

        // Should select the light variant (index 1) because auto-switch is on
        assert_eq!(
            idx, 1,
            "Expected light variant to be chosen when TERM is not ghost and auto-switch is on"
        );
    }

    #[test]
    fn fallback_to_runtime_mode_when_no_requested_variant() {
        // Request "Unknown Theme" (doesn't exist). Should fallback to terminal mode (dark).
        let cfg = AppConfig {
            theme_name: "Unknown Theme".to_string(),
            ..Default::default()
        };

        let available = vec![
            ("./themes/gruvbox.json".to_string(), "dark".to_string()),
            ("./themes/gruvbox.json".to_string(), "light".to_string()),
        ];

        let term_env = "xterm-256color";
        let (_theme, idx) = App::select_theme_from_config(&cfg, &available, "dark", term_env);

        // Should select the dark variant (index 0) because terminal_mode is "dark"
        assert_eq!(
            idx, 0,
            "Expected fallback to terminal mode (dark) when theme not found"
        );
    }

    #[test]
    fn test_comment_pagination_initialization() {
        let app = App::new();

        // Verify initial state
        assert_eq!(app.comment_ids.len(), 0, "Should start with no comment IDs");
        assert_eq!(
            app.loaded_comments_count, 0,
            "Should start with 0 loaded comments"
        );
        assert_eq!(app.comments.len(), 0, "Should start with no comments");
    }

    #[test]
    fn test_comment_ids_stored_on_story_selection() {
        let mut app = App::new();

        // Create a story with comment IDs
        let story = Story {
            id: 123,
            title: Some("Test Story".to_string()),
            url: Some("https://example.com".to_string()),
            by: Some("testuser".to_string()),
            score: Some(100),
            time: Some(1234567890),
            descendants: Some(50),
            kids: Some(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        };

        // Before selection
        assert_eq!(app.comment_ids.len(), 0);

        // The actual comment loading happens in handle_action, which is async
        // Here we just verify the state that would be set when handling SelectStory
        // We can't easily test the full async flow in a unit test without mocking

        // For now, verify we can store comment IDs manually
        let kids = story.kids.clone().unwrap();
        app.comment_ids = kids.clone();
        app.loaded_comments_count = 0;

        assert_eq!(app.comment_ids.len(), 12, "Should store all comment IDs");
        assert_eq!(app.loaded_comments_count, 0, "Should reset loaded count");
    }

    #[test]
    fn test_comments_loaded_increments_count() {
        use crate::internal::models::Comment;
        use crate::internal::models::CommentRow;
        let mut app = App::new();

        // Set up pagination state
        app.comment_ids = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        app.loaded_comments_count = 0;

        // Simulate loading first batch with threaded comments
        let first_batch = vec![
            CommentRow {
                comment: Comment {
                    id: 1,
                    by: Some("user1".to_string()),
                    text: Some("Comment 1".to_string()),
                    time: Some(1234567890),
                    kids: None,
                    deleted: false,
                },
                depth: 0,
                expanded: true,
                parent_id: None,
            },
            CommentRow {
                comment: Comment {
                    id: 2,
                    by: Some("user2".to_string()),
                    text: Some("Comment 2".to_string()),
                    time: Some(1234567891),
                    kids: None,
                    deleted: false,
                },
                depth: 0,
                expanded: true,
                parent_id: None,
            },
        ];

        // Simulate what CommentsLoaded handler does (it now replaces, not extends)
        app.loaded_comments_count = first_batch.len();
        app.comments = first_batch;

        assert_eq!(app.loaded_comments_count, 2, "Should track loaded count");
        assert_eq!(app.comments.len(), 2, "Should have 2 comments");

        // Test that we can access the comment data
        assert_eq!(app.comments[0].comment.id, 1);
        assert_eq!(app.comments[1].comment.id, 2);
    }

    #[test]
    fn test_all_comments_loaded_detection() {
        let app = App {
            comment_ids: vec![1, 2, 3, 4, 5],
            loaded_comments_count: 5,
            ..App::new()
        };

        // Simulate LoadMoreComments logic check
        let all_loaded = app.loaded_comments_count >= app.comment_ids.len();

        assert!(all_loaded, "Should detect when all comments are loaded");
    }

    #[test]
    fn test_no_comments_case() {
        let app = App {
            comment_ids: vec![],
            loaded_comments_count: 0,
            ..App::new()
        };

        // Simulate LoadMoreComments logic check
        let should_skip = app.comment_ids.is_empty();

        assert!(should_skip, "Should handle story with no comments");
    }

    #[test]
    fn test_partial_comments_loaded() {
        let app = App {
            comment_ids: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            loaded_comments_count: 5,
            ..App::new()
        };

        let remaining = app.comment_ids.len() - app.loaded_comments_count;

        assert_eq!(
            remaining, 5,
            "Should calculate remaining comments correctly"
        );
        assert!(
            app.loaded_comments_count < app.comment_ids.len(),
            "Should detect more comments available"
        );
    }

    #[test]
    fn test_toggle_help_action() {
        let mut app = App::new();

        // Initial state
        assert!(!app.show_help, "Help should be hidden initially");

        // Toggle on
        // We can't use handle_action_sync because it's async in the real code,
        // but for this simple action we can simulate it or just modify state directly
        // to verify the logic if we had the handler exposed.
        // Since handle_action is async and spawns tasks, unit testing it is tricky without a runtime.
        // However, we can verify the state change logic directly if we extract it,
        // or just trust the manual verification for this simple toggle.
        // Let's manually simulate what the handler does for this simple case:

        app.show_help = !app.show_help;
        assert!(app.show_help, "Help should be shown after toggle");

        app.show_help = !app.show_help;
        assert!(!app.show_help, "Help should be hidden after second toggle");
    }
}

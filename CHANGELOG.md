# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.2] - 2025-11-30

### Changed
- **Async API Migration**: Converted entire API service from blocking to async
  - Changed from `reqwest::blocking::Client` to async `reqwest::Client`
  - All API methods now async: `fetch_story_ids`, `fetch_story_content`, `fetch_comment_content`, `fetch_comment_tree`, `fetch_article_content`
  - Updated retry mechanism to use `tokio::time::sleep` instead of `std::thread::sleep`
  - Non-blocking API calls improve UI responsiveness
  - Foundation for future concurrent fetching optimizations

### Technical
- Removed `blocking` feature from reqwest dependency
- Fixed recursive async function with `Box::pin` and `Send` bound
- Updated all App integration points to use `.await` on API calls
- Converted all integration tests to `#[tokio::test]`
- Converted all unit tests to async with `mockito::Server::new_async()`
- Updated 7 API call sites in App with `.await`

### Testing
- ✅ All 77 tests passing (unit + integration + snapshot)
- ✅ Build verified successfully

## [0.7.1] - 2025-11-29

### Added
- **Integration Tests**: API integration testing with `mockito`
  - Mock API server setup for reliable testing without network calls
  - Tests for story list fetching (Top/New/etc.)
  - Tests for story details and comment fetching
  - `tests/api_integration.rs` with 2+ integration tests
- **Snapshot Tests**: UI rendering verification with `insta`
  - Snapshot testing for Log Viewer rendering
  - `tests/rendering.rs` with visual regression detection
  - Snapshots stored in `tests/snapshots/` directory
- **CI/CD Pipeline**: GitHub Actions workflow for automated testing
  - `.github/workflows/ci.yml` with multi-platform support
  - Automated formatting checks (`cargo fmt --check`)
  - Automated linting (`cargo clippy`)
  - Automated test execution (`cargo test`)
  - Dependency caching for faster builds

### Changed
- `ApiService::base_url` now exposed (not restricted to `#[cfg(test)]`) for integration testing
- `ApiService::with_base_url()` helper method now public for test usage

### Technical
- Added dev-dependencies: `mockito` (1.7.0), `insta` (1.44.3)
- Total test coverage: 75+ tests (unit + integration + snapshot)
- CI pipeline runs on Linux (Ubuntu) with full test suite

## [0.7.0] - 2025-11-29

### Added
- **Enhanced Notifications**: Color-coded user-facing notifications
  - `NotificationType` enum with Info, Warning, and Error variants
  - Auto-dismiss with configurable timeouts (Info: 3s, Warning: 5s, Error: 10s)
  - Color-coded notification overlay (Blue for Info, Yellow for Warning, Red for Error)
  - Notification helper methods: `notify_info()`, `notify_warning()`, `notify_error()`
- **Retry Mechanism**: Network resilience for API failures
  - Exponential backoff for transient failures
  - Configurable retry count and delays via `NetworkConfig` in `config.ron`
  - Smart retry only on network errors and timeouts (not on 4xx responses)
  - Retry progress logging with attempt counts
  - Configuration options: `max_retries`, `initial_retry_delay_ms`, `max_retry_delay_ms`, `retry_on_timeout`
- **Configurable Logging**: Flexible logging system
  - `LogConfig` in `config.ron` for log level configuration
  - Module-specific log levels via `module_levels` HashMap
  - Custom log directory support (defaults to `logs/`)
  - `RUST_LOG` environment variable takes precedence over config
  - `enable_performance_metrics` flag for conditional instrumentation
  - `LogLevel` enum: Trace, Debug, Info, Warn, Error
- **Log Viewer**: In-app debug log viewer
  - Toggle with `L` for debugging without leaving the app
  - Reads last 1000 lines from `logs/tui-hn-app.log`
  - Syntax highlighting for log levels (ERROR: red, WARN: yellow, INFO: blue, DEBUG: green, TRACE: magenta)
  - Scrollable log history (j/k/↑/↓ to scroll, G for bottom)
  - Close with Esc or q
  - Auto-scrolls to bottom when opened
- **Performance Metrics**: Comprehensive instrumentation
  - `#[tracing::instrument]` on all key functions
  - API request timing with conditional logging
  - Theme loading performance tracking
  - Cache operation timing (hit/miss tracking)
  - Debug-only rendering metrics (only in debug builds when `enable_performance_metrics` is enabled)
  - Per-view render timing (list, detail, article, bookmarks, history)
  - Overall draw function timing

### Changed
- `ApiService` now takes `enable_performance_metrics` parameter for conditional logging
- All `Cache` instances use `Cache::with_metrics()` for performance tracking
- `load_theme()` function signature updated to accept `enable_performance_metrics` flag
- `App::new` and `select_theme_from_config` now instrumented with tracing
- `handle_action` instrumented for action timing
- View rendering (`draw` function) instrumented with per-view timing

### Technical
- Added `notification` module with `Notification` and `NotificationType`
- Added `NetworkConfig` to `AppConfig` with retry configuration
- Added `LogConfig` and `LogLevel` to `AppConfig`
- Dynamic `EnvFilter` construction from config in `main.rs`
- Added `log_viewer` module with `LogViewer` struct and log parsing
- Pattern matching improvements in `detect_terminal_mode` and theme selection
- Conditional performance logging throughout `ApiService` and view rendering
- Added timing instrumentation to fetch methods with `std::time::Instant`

### Notes
- **Graceful Degradation (Phase 6)**: Deferred to pre-1.0 roadmap
  - Basic fallback already implemented (theme defaults, error notifications)
  - Additional fallback strategies for story/comment/article loading deferred
  - See TODO.md for pre-1.0 tracking



### Added
- **Status Bar Token Parsing**: Customizable status bar with format tokens
  - Token support: `{mode}`, `{category}`, `{count}`, `{total}`, `{sort}`, `{order}`, `{search}`, `{spinner}`, `{theme}`, `{shortcuts}`
  - Configure via `config.ui.status_bar_format` in `config.ron`
  - Real-time token replacement based on app state
  - Falls back to default behavior if not configured
- **List View Field Visibility**: Show/hide individual fields in story list
  - Configurable fields: score, comments, domain, age, author
  - Configure via `config.ui.list_view` settings
  - Smart separator handling based on visible fields
- **Complete Padding Customization**: All UI components now use configurable padding
  - Converted 7 remaining hardcoded padding locations
  - Consistent padding across story details, comments, articles, help overlay, and top bar

### Changed
- Extensive code quality improvements with pattern matching refactoring
  - Replaced if-else blocks with match expressions throughout codebase
  - Affected files: `app.rs`, `keybindings.rs`, `theme_editor.rs`, `view.rs`
  - More idiomatic Rust code with better exhaustiveness checking
- Status bar rendering now uses comprehensive match for all view modes
- List item rendering dynamically builds spans based on field visibility config

### Technical
- Added `parse_status_bar_format()` helper function for token parsing
- Improved list view rendering with `enumerate()` for proper indexing
- Better separation of concerns in status bar rendering logic

## [0.6.3] - 2025-11-29

### Added
- **Interactive Theme Editor**: Real-time theme customization
  - Toggle with `E` key
  - Visual overlay with property list and RGB sliders
  - Real-time preview of changes across the entire UI
  - Export custom themes to JSON (`s` key)
  - Keyboard-driven navigation (`↑`/`↓` properties, `←`/`→` channels, `+`/`-` adjust)
- **Multi-page Help System**:
  - Tab key toggles between General Shortcuts and Theme Editor Shortcuts
  - Updated help overlay layout

### Changed
- Improved help overlay with cleaner layout and better visibility for shortcuts
- Combined theme editor status and overlay for a unified experience

## [0.6.2] - 2025-11-28

### Added
- **Theme Editor Infrastructure**: Core backend for theme editing
  - `ThemeEditor` struct and state management
  - `ThemeProperty` and `ColorChannel` enums
  - Theme export functionality (`export_theme_to_file`)
  - `ToggleThemeEditor` and `ExportTheme` actions
- Updated `App` to support theme editing state

### Technical
- Added `serde_json` serialization for `TuiTheme` export
- Cleaned up `cargo clippy` warnings



## [0.6.1] - 2025-11-28

### Added
- **UI Customization (Initial)**: Configurable padding for UI elements
  - `padding` config option with `horizontal` and `vertical` settings
  - Applied to most UI components (list view, detail view, article view, status bar)
  - Configuration structures for future status bar format and list view customization
  - Examples and documentation in `config.example.ron`

### Technical
- Added `UIConfig`, `PaddingConfig`, and `ListViewConfig` to `AppConfig`
- Updated `view.rs` to use configurable padding instead of hardcoded values

## [0.6.0] - 2025-11-28

### Added
- **Key Binding Customization**: Full support for custom keybindings via `config.ron`
  - Define custom key mappings for all actions
  - Global and per-view mode keybindings (List, StoryDetail, Article, Bookmarks, History)
  - Hierarchical resolution: context-specific bindings override global bindings
  - Support for simple keys, special keys (Enter, Esc, Tab, etc.), and modifiers (Ctrl, Shift, Alt)
  - Examples and full documentation in `config.example.ron`
- Unit tests for keybinding resolution and parsing

### Changed
- Refactored `handle_normal_input()` to use keybinding system instead of hardcoded match statements
- Fixed regression: `q`/`Esc` now correctly quit in List view (was incorrectly going back)

### Technical
- Added `keybindings` module with `KeyBindingMap` and `KeyBindingContext`
- Added `keybindings_default` module with centralized default keybindings
- Custom `Serialize`/`Deserialize` implementation for `Action` enum
- Added `KeyBindingConfig` to `AppConfig` for config file support
- Added `Serialize`/`Deserialize` to `StoryListType` enum

## [0.5.3] - 2025-11-28

### Added
- **Sorting Options**: Sort stories by Score, Comments, or Time
  - Toggle sort order (Ascending/Descending) with `O` key
  - Shortcuts: `S` (Score), `C` (Comments), `T` (Time)
  - Visual indicator in list title showing current sort mode
- Unit tests for sorting logic

## [0.5.2] - 2025-11-28

### Added
- **Search Enhancements**: Enhanced search functionality
  - Regex search support with `Ctrl+R` or `F3`
  - Search mode switching (Title/Comments/Both) with `Ctrl+M` or `F2`
  - Search history navigation with `↑`/`↓` arrows
  - Persistent search history (last 20 searches)
  - Enhanced search overlay showing current mode and type
  - Regex error display in search overlay
- Unit tests for search functionality

### Changed
- Search UI now shows search mode and type in title bar
- Status bar in search mode shows new keyboard shortcuts
- All if-else blocks converted to pattern matching for consistency

### Technical
- Added `search` module with `SearchQuery`, `SearchMode`, `SearchType`, and `SearchHistory`
- Search history persisted to `search_history.json`
- Comment search currently limited to already-loaded/cached comments

## [0.5.1] - 2025-11-28

### Added
- **History Tracking**: Track recently viewed stories
  - Automatically saves viewed stories to `history.json`
  - View history with `H` key
  - Clear history with `X` key (in History view)
  - Tracks last 50 viewed stories
  - Shows "viewed X ago" timestamp
- Unit tests for history logic

### Technical
- Added `history` module with `History` and `ViewedStory` structs
- New `ViewMode::History`
- New actions: `ViewHistory`, `ClearHistory`

## [0.5.0] - 2025-11-28

### Added
- **Bookmarks/Favorites System**: Save stories for later reading
  - Toggle bookmark with `b` key on any story
  - View all bookmarks with `B` key
  - Bookmark indicator (★) displayed next to bookmarked stories
  - Persistent storage in `~/.config/tui-hn-app/bookmarks.json`
  - Dedicated bookmarks view with story count
  - Export bookmarks to JSON file
  - Import bookmarks from disk
  - Uses `jiff` for timestamp management

### Changed
- Enhanced story list rendering to support bookmark indicators
- Status bar now shows bookmark count when in bookmarks view
- Help screen updated with bookmark shortcuts

### Technical
- Added `bookmarks` module with `Bookmarks` and `BookmarkedStory` structs
- New `ViewMode::Bookmarks` for dedicated bookmarks view
- New actions: `ToggleBookmark`, `ViewBookmarks`, `ExportBookmarks`, `ImportBookmarks`
- Added dependencies: `jiff` (v0.2.16), `dirs` (v6.0.0), `serde_json` (v1.0.145)
- Pattern matching refactoring for improved code quality

## [0.4.2] - 2025-11-27

### Added
- **Story Metadata Display**: Enhanced story list view
  - Domain/source extraction and display (e.g., "github.com")
  - Two-line layout for better readability
  - Age indicator using relative time (already implemented in v0.3.x)

## [0.4.1] - 2025-11-27

### Added
- **Comment Threading**: Visualize comment hierarchy
  - Indentation for nested comments
  - Tree-like structure with visual guides (└─ and │)
  - Collapse/expand comment threads (togglable with Enter key)
  - Recursive comment fetching up to 100 comments

## [0.4.0] - 2025-11-27

### Added
- **Better Article Rendering**:
  - Rich text parsing using `scraper` (replacing `html2text`)
  - Syntax highlighting for code blocks
  - Table rendering (ASCII style)
  - Image placeholders
  - Improved list and quote styling

## [0.3.3] - 2025-11-27

### Added
- **Caching Layer**: In-memory cache with TTL to reduce API calls
  - Story cache (5 minute TTL)
  - Comment cache (5 minute TTL)
  - Article cache (15 minute TTL)
  - Thread-safe implementation using `Arc<RwLock<>>`

### Fixed
- Comment text wrapping width calculation for small terminals
- **Comment scrolling** — replaced List widget with Paragraph for smooth line-by-line scrolling
  - No more jumping between comments with empty space
  - j/k keys now scroll by single lines instead of full comments

## [0.3.2] - 2025-11-27

### Added
- **Keyboard Shortcuts Help**: Press `?` to show a help overlay with all available shortcuts
  - Categorized list (Global, Navigation, Story List, Article/Comments)
  - Context-aware closing (Esc/q/?)

## [0.3.1] - 2025-11-27

### Added
- **Animated Loading Spinner**: Smooth 10 FPS spinner animation using Unicode characters (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏)
  - Updates every 100ms in main event loop
  - Context-aware loading messages in status bar
  - Shows "⠋ Loading article/comments/stories..." based on active operation

### Changed
- **Enhanced Progress Overlay**: Improved "Load All Stories" progress display
  - Increased popup height for better visibility
  - Added animated spinner to title
  - Shows percentage as separate text (e.g., "9%")
  - Better layout with progress text, gauge, and percentage
- **Status Bar**: Replaced static "Loading..." with animated spinner and description
- **Code Quality**: Refactored `loading_description` to use pattern matching (more idiomatic Rust)

### Technical
- Added `spinner_state` and `last_spinner_update` to App state
- Added helper methods: `get_spinner_char()`, `active_loading_count()`, `loading_description()`

## [0.3.0] - 2025-11-27

### Added
- **Comment Pagination**: Load comments incrementally in batches of 20
  - Press `n` in Comments view to load more comments
  - Shows pagination status: "Comments (X/Y) - n: Load More"
  - Notification when all comments loaded
  - Initial load increased from 10 to 20 comments
- **Unit Tests**: Added 6 new tests for comment pagination logic
  - Test suite now includes 31 tests (up from 25)

### Changed
- Comments now append instead of replace when loading more
- Status bar in StoryDetail view updated to show `n: More Comments` hint

### Technical
- Added `comment_ids: Vec<u32>` to App state
- Added `loaded_comments_count: usize` to App state
- Added `LoadMoreComments` action to Action enum
- Improved code organization with comment pagination handlers

## [0.2.2] - 2025-11-26

### Added
- UI padding for better readability
  - Horizontal padding for top bar, status bar, list, detail, and article views
- App structure refactoring
  - Separated state management (`internal/ui/app.rs`) from rendering (`internal/ui/view.rs`)
  - Improved code organization and maintainability

### Changed
- Updated README screenshots version note to v0.2.2

## [0.2.1] - 2025-11-25

### Added
- Runtime theme auto-switch toggle (`g` key)
- Theme configuration persistence
- Auto-switch dark to light theme based on terminal mode
- Ghost terminal detection for explicit theme variant control

### Changed
- Improved theme selection logic
- Enhanced logging for theme debugging

## [0.2.0] - 2025-11-24

### Added
- Custom theme support with JSON files
- Theme switching at runtime (`t` key)
- Multiple theme variants (dark/light)
- Theme auto-discovery from `./themes` directory
- Configuration file (`config.ron`) support

### Changed
- Improved TUI styling and color scheme
- Enhanced status bar with theme information

## [0.1.0] - 2025-11-23

### Added
- Initial TUI Hacker News client
- Story browsing (Top, New, Best, Ask, Show, Job)
- Story detail view with comments
- Article content view (in-app rendering)
- Search/filter functionality
- Story pagination (load more/all)
- Open in browser functionality
- Keyboard navigation (vim-style and arrow keys)
- Dark theme support

### Technical
- Built with `ratatui` for terminal UI
- `tokio` for async runtime
- `reqwest` for API calls
- HTML to text conversion for articles

---

[0.3.0]: https://github.com/yourusername/tui-hn-app/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/yourusername/tui-hn-app/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/yourusername/tui-hn-app/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/yourusername/tui-hn-app/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/tui-hn-app/releases/tag/v0.1.0

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

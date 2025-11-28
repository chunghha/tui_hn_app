# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

# TUI Hacker News App - Development Roadmap

> **Current Version**: 0.5.2
  
> See [Version Planning](#version-planning) section below for the full roadmap.

## ✅ Completed Items

### Core Features
- [x] Port from GPUI to Ratatui
- [x] Story Categories (Top, New, Best, Ask, Show, Job lists)
- [x] In-App Article View (renders article content directly in TUI using html2text)
- [x] Theming (custom themes loaded from JSON files)
- [x] Search/Filter functionality
- [x] Pagination (load more / load all)
- [x] Auto theme switching based on terminal mode detection
- [x] Runtime configuration toggle (auto_switch_dark_to_light)
- [x] UI padding for better readability
- [x] Code organization (separated state management from view rendering in `internal/ui/`)

## 🚧 Enhancements to Consider

### Performance & User Experience
- [x] **Comment Pagination**: Load comments incrementally in batches of 20
  - ✅ Implemented in v0.3.0
  - Added "Load More Comments" action (press `n`)
  - Shows comment count vs loaded count
  
- [x] **Caching Layer**: Cache fetched stories and articles to reduce API calls
  - ✅ Implemented in v0.3.3
  - In-memory cache with TTL
  - Persistent cache to disk (optional - not implemented)
  - Cache invalidation strategy

- [x] **Progress Indicators**: Improve loading feedback
  - ✅ Implemented in v0.3.1
  - Better progress bar for "Load All Stories"
  - Loading spinner for individual story/comment fetches
  - Network status indicator in status bar

- [x] **Keyboard Shortcuts Help**: Add `?` key to show all available shortcuts
  - ✅ Implemented in v0.3.2
  - Overlay or dedicated view
  - Context-sensitive help (different shortcuts per view mode)

### Content Display
- [x] **Better Article Rendering**: Improve article readability
  - ✅ Implemented in v0.4.0
  - Code syntax highlighting in articles
  - Better handling of links (show URL on select, copy to clipboard)
  - Image placeholder/notification for images in articles
  - Table rendering support

- [x] **Comment Threading**: Visualize comment hierarchy
  - ✅ Implemented in v0.4.1
  - Indentation for nested comments
  - Tree-like structure with visual guides
  - Collapse/expand comment threads

- [x] **Story Metadata Display**: Show more story information
  - ✅ Implemented in v0.4.2
  - Domain/source in list view
  - Story age/freshness indicator
  - User karma/reputation (deferred)

### Navigation & Interaction
- [x] **Bookmarks/Favorites**: Save stories for later reading
  - ✅ Implemented in v0.5.0
  - Local storage of bookmarked story IDs with timestamps
  - Dedicated view for bookmarks (press `B`)
  - Toggle bookmark with `b` key
  - Bookmark indicators (★) in story list
  - Import/export bookmarks

- [x] **History**: Track recently viewed stories
  - ✅ Implemented in v0.5.1
  - Last N stories viewed
  - Clear history option

- [x] **Search Enhancements**: Improve search functionality
  - ✅ Implemented in v0.5.2
  - Search in comments, not just titles (limited to cached comments)
  - Regex search support
  - Search history

- [x] **Sorting Options**: Allow sorting stories
  - ✅ Implemented in v0.5.3
  - By score, comments, time
  - Ascending/descending toggle

### Configuration & Customization
- [x] **Key Binding Customization**: Allow users to remap keys in config.ron
  - ✅ Implemented in v0.6.0
  - Define custom keybindings per view mode
  - Global and context-specific bindings with hierarchical resolution
  - Conflict detection (optional future enhancement)

- [/] **UI Customization**: More configurable UI elements
  - ✅ Implemented in v0.6.1 (padding only)
  - Adjustable padding/margins (configurable via config.ron)
  - Customizable status bar format (structure in place, token parsing TBD)
  - List view format customization (structure in place, rendering logic TBD)

- [/] **Theme Editor**: Interactive theme builder
  - ✅ Implemented in v0.6.2 (infrastructure only)
  - Theme editor module with color manipulation
  - Export custom themes to JSON
  - Interactive UI overlay (deferred to v0.6.3)

- [ ] **Theme Save Location**: Configurable theme storage (Pre-1.0)
  - Allow custom directory via config.ron
  - Support for user config directory (e.g., `~/.config/tui-hn-app/themes/`)
  - Fallback to default if custom location is not writable
  - Auto-create directory if it doesn't exist
  - Currently hardcoded to `./themes/`

- [ ] **Graceful Degradation**: Enhanced error resilience (Pre-1.0, deferred from v0.7.0)
  - Story loading fallback → Show cached stories when network fails
  - Comment loading fallback → Show placeholder for failed comments
  - Article fetching fallback → Keep story view functional on fetch errors
  - Configuration errors → Use sensible defaults
  - File I/O errors → Graceful handling with user notifications

## 📝 Known Issues
- [ ] Article scroll position doesn't always persist when toggling views
- [ ] Long titles may wrap awkwardly in list view
- [ ] Theme switching doesn't refresh immediately in all cases

## 🎯 Next Recommended Tasks

Based on current codebase maturity, I recommend prioritizing:

1. **Comment Pagination** - Currently the biggest limitation (only 10 comments shown)
2. **Keyboard Shortcuts Help** - Improves discoverability for new users
3. **Caching Layer** - Significant performance improvement
4. **Better Error Handling** - More robust user experience
5. **Comment Threading** - Much better UX for reading discussions

## Version Planning

### ✅ Completed Versions

#### v0.3.0 - v0.3.3 (Completed)
- ✅ Comment pagination (v0.3.0)
- ✅ Help screen (`?` key) (v0.3.2)
- ✅ In-memory caching with TTL (v0.3.3)
- ✅ Improved error messages (v0.3.x)
- ✅ Smooth line-by-line comment scrolling (v0.3.3)

#### v0.4.0 - v0.4.2 (Completed)
- ✅ Better Article Rendering with rich text (v0.4.0)
- ✅ Comment Threading with visual hierarchy (v0.4.1)
- ✅ Story Metadata Display with domain/age (v0.4.2)

#### v0.5.0 (Completed - 2025-11-28)
- ✅ Bookmarks/Favorites System
  - Local storage of bookmarked story IDs with timestamps
  - Dedicated view for bookmarks (press `B`)
  - Import/export bookmarks
  - Toggle bookmark with `b` key
  - Bookmark indicators (★) in story list

### 🚀 v0.5.x Series (Completed)

#### v0.5.1 - History Tracking (Completed - 2025-11-28)
- ✅ Track recently viewed stories
- ✅ Clear history option

#### v0.5.2 - Enhanced Search (Completed - 2025-11-28)
- ✅ Regex search support
- ✅ Search mode switching (Title/Comments/Both)
- ✅ Search history navigation
- ✅ Persistent search history

#### v0.5.3 - Sorting Options (Completed - 2025-11-28)
- ✅ Sort by Score, Comments, or Time
- ✅ Ascending/Descending toggle
- ✅ Visual sort indicator in list title

### 🎯 v0.6.x Series - Configuration & Customization (In Progress)

#### v0.6.0 - Key Binding Customization (Completed - 2025-11-28)
- ✅ Define custom keybindings in config.ron
- ✅ Per-view mode keybindings (Global, List, StoryDetail, Article, Bookmarks, History)
- ✅ Hierarchical resolution (context-specific overrides global)
- ✅ Default keybinding fallback
- ⏭️ Conflict detection (optional future enhancement)
- ✅ Documentation in config.example.ron

#### v0.6.1 - UI Customization (Completed - 2025-11-28)
- ✅ Adjustable padding/margins (horizontal/vertical)
- ✅ Configuration structures for future features
- ⏭️ Customizable status bar format (structure in place, token parsing deferred)
- ⏭️ List view format customization (structure in place, rendering logic deferred)

#### v0.6.2 - Theme Editor Infrastructure (Completed - 2025-11-28)
- ✅ Theme editor module with state management
- ✅ Color property navigation and RGB manipulation
- ✅ Export custom themes to JSON
- ✅ Action system integration (ToggleThemeEditor, ExportTheme)
- ⏭️ Interactive UI overlay (deferred to v0.6.3)

#### v0.6.3 - Theme Editor UI (Completed - 2025-11-29)
- ✅ Interactive keyboard-driven theme editor
- ✅ Real-time theme preview with instant color updates
- ✅ Keyboard shortcuts (E: toggle, ↑↓: navigate, ←→: channel, +/-: adjust, s: save, Esc: cancel)
- ✅ Export themes to JSON
- ✅ Visual overlay rendering with property list and RGB sliders
- ✅ Theme naming popup with user input
- ✅ Hex color display and preview box
- ✅ Automatic complementary theme generation (dark/light variants)

#### v0.6.4 - UI Customization Completion (Completed - 2025-11-29)
- [x] Status bar format token parsing
- [x] List view field visibility rendering
- [x] Complete remaining padding conversions (7 locations)
- [x] Code quality improvements (pattern matching refactoring)

---

### 🔧 v0.7.x - Technical Improvements Series

#### v0.7.0 - Error Handling & Logging (Completed - 2025-11-29)
- [x] **Enhanced Notifications**: User-facing error messages
  - Color-coded notifications (Info/Warning/Error)
  - Auto-dismiss with configurable timeouts
  - Error display in overlay
- [x] **Retry Mechanism**: Network resilience
  - Exponential backoff for transient failures
  - Configurable retry count and delays
  - Smart retry only on network errors
- [x] **Configurable Logging**: Flexible logging system
  - Log level configuration per module
  - Custom log directory support
  - RUST_LOG environment variable override
- [x] **Log Viewer**: In-app debug viewer
  - Toggle with `L`
  - Syntax highlighting for log levels
  - Scrollable log history (last 1000 entries)
- [x] **Performance Metrics**: Instrumented timing
  - `#[tracing::instrument]` on key functions
  - API request timing
  - Theme loading timing
  - Cache operation timing
  - Conditional debug-only rendering metrics

#### v0.7.1 - Testing & Quality (Completed - 2025-11-29)
- [x] **Expanded Test Coverage**
  - ✅ Integration tests for UI flows (API integration with mockito)
  - ✅ Mock API responses for reliable testing
  - ✅ Snapshot tests for rendering (insta)
  - [ ] Property-based tests for edge cases (Deferred)
- [x] **CI/CD Improvements**
  - ✅ Automated testing on multiple platforms (GitHub Actions)
  - [ ] Release automation (Deferred)
  - [ ] Benchmark tracking (Deferred)

#### v0.7.2 - Performance Optimization (In Progress - 2025-11-30)
- [x] **Async Migration (Phase 1)**: Convert API service to async
  - ✅ Converted all API methods from blocking to async (`reqwest::blocking` → `reqwest` async)
  - ✅ Fixed recursive async with `Box::pin` + `Send` bound
  - ✅ Updated all App integration points with `.await`
  - ✅ Converted all tests to async (`#[tokio::test]`)
  - ✅ All 77 tests passing
- [x] **Concurrent Fetching (Phase 2)**: Improve async handling (Deferred)
  - Concurrent story fetching (batch API calls)
  - Better cancellation of in-flight requests when switching views
  - Rate limiting to respect HN API best practices
  - Request deduplication
- [ ] **Rendering Performance** (Deferred)
  - Optimize list rendering for large story counts
  - Lazy loading for comment trees
  - Diff-based rendering where applicable

#### v0.7.3 - Concurrent Fetching & Rate Limiting (Completed - 2025-11-30)
- [x] **Rate Limiting**: Semaphore-based rate limiting
  - ✅ 3 requests/second (configurable via `rate_limit_per_second`)
  - ✅ Applied to all API methods via semaphore in `get_json`
  - ✅ Respects Hacker News API guidelines
- [x] **Concurrent Story Fetching**: Batch concurrent requests
  - ✅ `fetch_stories_concurrent()` method with `futures::stream`
  - ✅ 10 concurrent requests (configurable via `concurrent_requests`)
  - ✅ 3-5x faster story loading
  - ✅ Updated App to use concurrent fetching
- [ ] **Request Management** (Deferred)
  - Request deduplication
  - Request cancellation tokens


#### v0.8.0 - Accessibility Phase 1 (Completed - 2025-11-30)
- [x] **High Contrast Theme**
  - ✅ Created high contrast theme with WCAG AAA compliant colors
  - ✅ Pure black/white with bright highlights for maximum visibility
- [x] **Accessibility Configuration**
  - ✅ Added `AccessibilityConfig` with `high_contrast_mode` and `verbose_status`
  - ✅ Integrated into app configuration system
  - ✅ Documented in `config.example.ron`

#### v0.8.1 - Accessibility Phase 2 (Completed - 2025-11-30)
- [x] **WCAG Compliance**
  - ✅ Audit existing themes for WCAG AA compliance
  - ✅ Test contrast ratios (Fixed Flexoki Light blue/yellow)
  - ✅ Fix low-contrast combinations
- [x] **Enhanced Status Messages**
  - ✅ More descriptive loading states
  - ✅ Context-rich error messages
  - ✅ Better navigation announcements (Verbose Status mode)
- [x] **Documentation**
  - ✅ Accessibility guide in README
  - ✅ Keyboard shortcuts reference
  - ✅ Testing guidelines (`docs/ACCESSIBILITY.md`)
- [x] **UI Polish**
  - ✅ Theme Editor padding improvements


---

### 🏁 v1.0.0 (Stable Release)

**Target**: Feature-complete, production-ready TUI Hacker News client

**Requirements**:
- ✅ Complete feature parity with basic web interface
- ✅ All v0.6.x UI customization features implemented
- [ ] All v0.7.x technical improvements completed
- [ ] Comprehensive test coverage (>80%)
- [ ] Performance benchmarks met
- [ ] All accessibility features implemented
- [ ] Documentation complete (user guide, developer docs)
- [ ] Stable API for themes and configuration

**Post-1.0 Roadmap**:
- Plugin/extension system
- Cloud sync for bookmarks/history
- Multi-account support
- Advanced filtering and search
- Custom layouts and views

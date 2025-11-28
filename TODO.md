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

- [ ] **UI Customization**: More configurable UI elements
  - Adjustable padding/margins
  - Customizable status bar format
  - List view format customization (show/hide fields)

- [ ] **Theme Editor**: Interactive theme builder
  - Preview theme changes in real-time
  - Export custom themes

### Technical Improvements
- [ ] **Error Handling**: Better user-facing error messages
  - Show network errors in notification
  - Retry mechanism for failed requests
  - Fallback strategies

- [ ] **Logging**: Better tracing and debugging
  - Log rotation configuration in config.ron
  - Different log levels per module
  - Log viewer in app (debug mode)

- [ ] **Testing**: Expand test coverage
  - Integration tests for UI flows
  - Mock API responses for reliable testing
  - Snapshot tests for rendering

- [ ] **Async Optimization**: Improve async handling
  - Concurrent story fetching (batch API calls)
  - Better cancellation of in-flight requests when switching views
  - Rate limiting to respect HN API best practices

### Accessibility
- [ ] **Screen Reader Support**: Improve accessibility
  - Better text descriptions
  - Announce loading states

- [ ] **High Contrast Themes**: Built-in high contrast mode

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

### 🎯 v0.6.0 Series (In Progress)

#### v0.6.0 - Key Binding Customization
- [ ] Define custom keybindings in config.ron
- [ ] Per-view mode keybindings
- [ ] Conflict detection
- [ ] Default keybinding fallback

#### v0.6.1 - UI Customization
- [ ] Adjustable padding/margins
- [ ] Customizable status bar format
- [ ] List view format customization (show/hide fields)
- [ ] Configurable colors for UI elements

#### v0.6.2 - Theme Editor (Optional)
- [ ] Interactive theme builder
- [ ] Preview theme changes in real-time
- [ ] Export custom themes

### 🏁 v1.0.0 (Stable)
- Complete feature parity with web interface
- Comprehensive test coverage
- Performance optimizations
- All accessibility features


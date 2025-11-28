# TUI Hacker News App - Development Roadmap

> **Current Version**: 0.3.3 (in development)  
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

- [ ] **Story Metadata Display**: Show more story information
  - Domain/source in list view
  - Story age/freshness indicator
  - User karma/reputation if available

### Navigation & Interaction
- [ ] **Bookmarks/Favorites**: Save stories for later reading
  - Local storage of bookmarked story IDs
  - Dedicated view for bookmarks
  - Import/export bookmarks

- [ ] **History**: Track recently viewed stories
  - Last N stories viewed
  - Clear history option

- [ ] **Search Enhancements**: Improve search functionality
  - Search in comments, not just titles
  - Regex search support
  - Search history

- [ ] **Sorting Options**: Allow sorting stories
  - By score, comments, time
  - Ascending/descending toggle

### Configuration & Customization
- [ ] **Key Binding Customization**: Allow users to remap keys in config.ron
  - Define custom keybindings per view mode
  - Conflict detection

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

### v0.3.0 (Next Minor Release)
- Comment pagination
- Help screen (`?` key)
- Basic caching
- Improved error messages

### v0.4.0 (Future)
- Comment threading/indentation
- Bookmarks/favorites
- History tracking

### v1.0.0 (Stable)
- Complete feature parity with web interface
- Comprehensive test coverage
- Performance optimizations
- All accessibility features

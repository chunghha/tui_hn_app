# Port GPUI HN App to Ratatui

## Goal Description
Port the existing Hacker News application from `gpui` to `ratatui` to create a Terminal User Interface (TUI) version, while preserving the core application logic.

## User Review Required
- [ ] Confirmation of key bindings scheme (vim-like vs arrow keys).
- [ ] Visual design preference for TUI (borders, colors).

## Proposed Changes

### Dependencies
- Remove `gpui`, `gpui-component`.
- Add `ratatui`, `crossterm`, `tokio` (if not present/compatible).

### Core Logic
- **Refactor `AppState`**:
    - Remove `gpui::Entity`, `gpui::App`, `gpui::Context` dependencies.
    - Use `tokio` channels (`mpsc`) for async communication between API fetching tasks and the main UI thread.
    - Store state in a struct that can be mutated by the main event loop.
- **Async Runtime**:
    - Switch to `tokio` runtime for the main function.
    - Use `tokio::spawn` for background API requests.

### UI Layer
- **TUI Framework**: Use `ratatui` for rendering.
- **Event Handling**: Use `crossterm` for input events.
- **Components**:
    - `StoryList`: Render list of stories using `ratatui::widgets::List`.
    - `StoryDetail`: Render story details and comments using `ratatui::widgets::Paragraph` and custom layout.
    - `Status Bar`: Show current mode, loading status, etc.
- **Navigation**:
    - `j`/`k` or Up/Down for list navigation.
    - `Enter` to view story details.
    - `o` to open in browser.
    - `Esc` or `q` to go back/quit.

## New Features (Enhancements)
- [x] **Story Categories**: Support Top, New, Best, Ask, Show, Job lists.
- [ ] **In-App Article View**: Render article content directly in the TUI.
- [ ] **Theming**: Support custom themes loaded from JSON files.

## Verification Plan
### Automated Tests
- Run existing unit tests for logic.
- Add new tests for TUI state transitions.

### Manual Verification
- Launch the app in terminal.
- Navigate story list.
- Open a story and view comments.
- Verify "load more" functionality.
- Verify category switching.
- Verify article reading.
- Verify theme switching.

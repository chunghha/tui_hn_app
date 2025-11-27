# TUI Hacker News App (ratatui)

A terminal-based Hacker News client written in Rust using `ratatui`. This project provides a fast, keyboard-driven TUI for browsing Hacker News stories, viewing story details and comments, and reading articles inside an embedded terminal view.

This README documents the current TUI-focused implementation and recent UI improvements (theme location, version placement, and article-view behavior).

## Architecture

This is a single-binary Rust TUI app structured for clarity between public API code, services, and UI rendering.

- Core TUI rendering: `ratatui`
- Async tasks & channels: `tokio`
- HTTP API client: `reqwest` (blocking client is used in the API service to simplify testing)
- Theming: JSON theme files parsed into a `TuiTheme` used by `ratatui` styles
- Configuration: RON (`config.ron`) with `AppConfig`

### High-level project structure

```
src/
├── api/              # Hacker News API service and types (fetching stories, comments, articles)
├── config.rs         # Configuration loading and management (AppConfig)
├── internal/         # Internal implementation modules and models
│   ├── models.rs     # Data models (Story, Comment)
│   └── ...           # internal helpers
├── lib.rs            # Library entry point (public module exports)
├── main.rs           # Application entry point (init, run)
├── tui.rs            # Terminal init/restore helpers (enter/leave alternate screen)
└── utils/            # Utility functions (datetime, html extraction, theme loader)
```

## Features

- Browse Hacker News categories: Best, Top, New, Ask, Show, Job
- Keyboard-driven navigation (vi-like keys)
- Story details and comments view
- Inline article viewing (fetched and rendered as plain text)
- Theme loading from JSON files; theme preview in top-right header
- Version shown in the list title to make builds traceable from the UI
- Search/filter stories in the list
- Incremental loading with "Load More" and "Load All" behaviors

## Screenshots

(These are illustrative — your terminal size and theme will affect appearance.)

| List View  | Article View | Comments View |
|:---:|:---:|:---:|
| ![List View](screenshots/list_view.png)  | ![Article View](screenshots/article_view.png) | ![Comments View](screenshots/comments_view.png) |
| Hacker News Category List | Article content for a selected story | Comments for the selected story |

Note: These screenshots were taken with version `v0.1.0`. Subsequent UI enhancements were made after that release, so the current app appearance may differ from the images shown here.

## Configuration

The app reads configuration from `config.ron` (searched in the working directory and next to the executable). A `config.example.ron` is provided — copy it to `config.ron` and edit as needed.

Important config keys:
- `font_sans`, `font_serif`, `font_mono` — used conceptually; terminal fonts are controlled by your terminal emulator.
- `theme_name` — preferred theme name (e.g., `"Flexoki Light"` / `"Flexoki Dark"`).
- `theme_file` — path to themes directory or specific theme JSON.
- `webview_zoom` — initial article-view width handling (conceptual for layout).
- `soft_wrap_max_run` — pre-processing for markdown/soft-wrap behavior.

Example (abbreviated):
```text
# copy config.example.ron -> config.ron and edit
theme_name: "Flexoki Light"
theme_file: "./themes"
webview_zoom: 120
```

## Usage

Run locally from the project root:

- Development run:
  - `cargo run`
- Recommended local workflow (TDD-friendly):
  - `task fmt`     — runs `cargo fmt`
  - `task clippy`  — runs `cargo clippy`
  - `task build`   — runs format + clippy then `cargo build --release`
  - `task run`     — runs the release binary

If you prefer direct cargo commands:
- Format: `cargo fmt`
- Lint: `cargo clippy --all-targets --all-features`
- Run: `cargo run`

## Keyboard Shortcuts

- `j` / `k` — navigate down/up through lists
- `Enter` — open selected story (detail view)
- `Tab` — toggle between Comments and Article view (Article view fetches the article for the currently selected story)
- `/` — enter search mode to filter story list
- `1..6` — switch story categories (Top, New, Best, Ask, Show, Job)
- `t` — switch theme (cycles through loaded theme variants)
- `q` or `Esc` — back / quit depending on the view
- `m` — load more stories (pagination)
- `A` — load all remaining stories (progress overlay shown)

## Behavior notes / UX details

- Title location: The list title now includes the app version (from `CARGO_PKG_VERSION`), making it easy to confirm which build is running.
- Theme location: Theme name and variant appear right-aligned in the top bar.
- Article fetch logic: Selecting a new story clears any previously fetched article content; toggling to Article view triggers a fresh fetch for the active story. This avoids showing stale article content when switching selection.

## Theming

- Drop JSON theme files into `./themes`. The app discovers themes and will list available variants (dark/light).
- You can cycle themes with `t`. The active theme name is shown top-right.

## Testing

- Unit tests exist for API helpers and utilities (run with `cargo test`).
- Tests are written to avoid network dependency where possible (mockito is used for `ApiService` tests).

## Contributing

We follow a TDD-first and tidy-first workflow:
1. Write the smallest failing test for the behavior (Red).
2. Implement the minimal code to make the test pass (Green).
3. Refactor and tidy without changing behavior, run tests (Tidy First).
4. Keep commits small and focused; run `cargo fmt` and `cargo clippy` before committing.

See `AGENTS.md` in the repo for more detail on the development discipline and preferred commands.

## Troubleshooting

- If the terminal UI looks off, ensure your terminal emulator supports true color and uses a sufficient font size/width.
- If the article view appears empty, check your network connectivity — the app fetches article content from the URL and converts HTML to text.
- For theme problems, validate theme JSON syntax and ensure theme names match the `theme_name` value in `config.ron`.

---

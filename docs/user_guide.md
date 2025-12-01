# TUI Hacker News App - User Guide

Welcome to the TUI Hacker News App! This guide will help you install, configure, and use the application efficiently.

## Installation

### From Source
```bash
git clone https://github.com/chunghha/tui_hn_app.git
cd tui_hn_app
cargo install --path .
```

### From Release
Download the latest binary for your platform from the [Releases](https://github.com/chunghha/tui_hn_app/releases) page.

## Configuration

The application uses a configuration file located at:
- **Linux**: `~/.config/tui-hn-app/config.ron`
- **macOS**: `~/Library/Application Support/tui-hn-app/config.ron`
- **Windows**: `%APPDATA%\tui-hn-app\config.ron`

### Example Config
```ron
(
    // Theme selection
    theme_name: "Flexoki Dark",
    
    // Custom theme directory
    theme_directory: "/Users/you/.config/tui-hn-app/themes",

    // Keybindings
    keybindings: (
        global: {
            "q": Quit,
            "?": ToggleHelp,
        },
        list: {
            "j": NavigateDown,
            "k": NavigateUp,
            "enter": Enter,
        },
        // ... see README for full list
    ),
)
```

## Keybindings Cheat Sheet

### Global
- `q`: Quit
- `?`: Toggle Help
- `Esc`: Back / Cancel

### Navigation
- `j` / `Down`: Move down
- `k` / `Up`: Move up
- `g`: Go to top
- `G`: Go to bottom

### Story List
- `Enter`: View comments
- `o`: Open in browser
- `s`: Toggle sort order (Top/New/Best/etc.)
- `r`: Refresh list

### Comments
- `Enter`: Toggle collapse
- `l`: Load more comments

## Themes

You can create custom themes by placing JSON files in your `theme_directory`.
Press `t` in the app to open the Theme Editor and create your own!

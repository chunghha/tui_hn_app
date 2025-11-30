# Accessibility Guide

This guide describes the accessibility features of the TUI Hacker News App and how to use them.

## Features

### High Contrast Theme
We provide a WCAG AAA compliant High Contrast theme designed for maximum visibility.
- **Background**: Pure Black (#000000)
- **Text**: Pure White (#FFFFFF)
- **Highlights**: Bright Yellow (#FFFF00)

To enable:
1. Press `t` to open the Theme Selector.
2. Select "High Contrast".
3. Press `Enter`.

Alternatively, set `high_contrast_mode = true` in your `config.ron`.

### Verbose Status Messages
For screen reader users, we offer a "Verbose Status" mode that replaces the compact status bar with descriptive sentences.
- **Standard**: `Top | 20/500 | ?`
- **Verbose**: `Viewing Top Stories. 20 stories loaded. Press Question Mark for help.`

To enable, set `verbose_status = true` in your `config.ron` under the `accessibility` section.

## Keyboard Shortcuts

### Global
- `?`: Toggle Help Overlay
- `q`: Quit (or Back)
- `Esc`: Back / Cancel
- `t`: Switch Theme
- `b`: Bookmark Story
- `B`: View Bookmarks
- `H`: View History

### List View
- `j` / `Down`: Next Story
- `k` / `Up`: Previous Story
- `Enter`: View Story Details
- `m`: Load More Stories
- `A`: Load All Stories (use with caution)
- `/`: Search
- `1`-`6`: Switch List (Top, New, Best, Ask, Show, Job)

### Story Detail View
- `Tab`: Switch to Article View (Reader Mode)
- `n`: Load next batch of comments
- `o`: Open in Browser

### Article View
- `j` / `k`: Scroll Article
- `Esc`: Return to Story Details

## Screen Reader Tips
- Use a terminal emulator with good screen reader integration (e.g., Terminal.app with VoiceOver on macOS).
- Enable `verbose_status` for better context.
- Use the High Contrast theme if you have low vision.

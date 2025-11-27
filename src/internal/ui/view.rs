use std::path::Path;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Alignment;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap};
use textwrap;

use super::app::{App, InputMode, ViewMode};

pub fn draw(app: &mut App, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_top_bar(app, f, chunks[0]);

    match app.view_mode {
        ViewMode::List => render_list(app, f, chunks[1]),
        ViewMode::StoryDetail => render_detail(app, f, chunks[1]),
        ViewMode::Article => render_article(app, f, chunks[1]),
    }

    render_status_bar(app, f, chunks[2]);

    // Render search overlay if in search mode
    if app.input_mode == InputMode::Search {
        render_search_overlay(app, f);
    }

    // Render notification overlay if present
    if app.notification_message.is_some() {
        render_notification(app, f);
    }

    // Render progress overlay if loading all stories
    if app.story_load_progress.is_some() {
        render_progress_overlay(app, f);
    }

    // Render help overlay if active
    if app.show_help {
        render_help_overlay(app, f);
    }
}

fn render_progress_overlay(app: &App, f: &mut Frame) {
    if let Some((loaded, total)) = app.story_load_progress {
        let area = f.area();
        let popup_width = 60.min(area.width - 4);
        let popup_height = 5; // Reduced height
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Add spinner to title
        let spinner = app.get_spinner_char();
        let percent = if total > 0 {
            (loaded as f64 / total as f64 * 100.0) as u16
        } else {
            0
        };

        let block = Block::default()
            .title(format!("{} Loading all stories", spinner))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border))
            .style(Style::default().bg(app.theme.background));

        f.render_widget(Clear, popup_area);
        f.render_widget(&block, popup_area);

        let inner_area = block.inner(popup_area);

        // Split into sections for gauge and percentage
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Gauge
                Constraint::Length(1), // Percentage
            ])
            .split(inner_area);

        // Gauge with label
        let gauge = ratatui::widgets::Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(app.theme.selection_bg)
                    .bg(app.theme.background),
            )
            .label(format!("{}/{}", loaded, total))
            .percent(percent);
        f.render_widget(gauge, chunks[0]);

        // Percentage text
        let percent_text = Paragraph::new(format!("{}%", percent))
            .style(Style::default().fg(app.theme.comment_time))
            .alignment(Alignment::Center);
        f.render_widget(percent_text, chunks[1]);
    }
}

fn render_list(app: &mut App, f: &mut Frame, area: Rect) {
    // Filter stories based on search query
    let filtered_stories: Vec<_> = if app.search_query.is_empty() {
        app.stories.iter().enumerate().collect()
    } else {
        let query = app.search_query.to_lowercase();
        app.stories
            .iter()
            .enumerate()
            .filter(|(_, story)| {
                story
                    .title
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&query))
                    .unwrap_or(false)
            })
            .collect()
    };

    let items: Vec<ListItem> = filtered_stories
        .iter()
        .map(|(_, story)| {
            let title = story.title.as_deref().unwrap_or("No Title");
            let score = story.score.unwrap_or(0);
            let by = story.by.as_deref().unwrap_or("unknown");
            let comments = story.descendants.unwrap_or(0);

            let time = story
                .time
                .as_ref()
                .map(crate::utils::datetime::format_timestamp)
                .unwrap_or_else(|| "unknown".to_string());

            let content = Line::from(vec![
                Span::styled(format!("{} ", score), Style::default().fg(app.theme.score)),
                Span::styled(title, Style::default().fg(app.theme.foreground)),
                Span::styled(
                    format!(" ({} comments by {} | {})", comments, by, time),
                    Style::default().fg(app.theme.comment_time),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    // Place the version next to the "Hacker News" label in the title
    let title = if app.search_query.is_empty() {
        format!(
            "Hacker News v{} - {}",
            app.app_version, app.current_list_type
        )
    } else {
        format!(
            "Hacker News v{} - {} (Filter: {})",
            app.app_version, app.current_list_type, app.search_query
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1))
                .border_style(Style::default().fg(app.theme.border))
                .title(title)
                .title_style(Style::default().fg(app.theme.foreground)),
        )
        .style(Style::default().bg(app.theme.background))
        .highlight_style(
            Style::default()
                .bg(app.theme.selection_bg)
                .fg(app.theme.selection_fg)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, area, &mut app.story_list_state);
}

fn render_detail(app: &mut App, f: &mut Frame, area: Rect) {
    if let Some(story) = &app.selected_story {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        let title = story.title.as_deref().unwrap_or("No Title");
        let url = story.url.as_deref().unwrap_or("No URL");
        let time = story
            .time
            .as_ref()
            .map(crate::utils::datetime::format_timestamp)
            .unwrap_or_else(|| "unknown".to_string());
        let text = format!(
            "Title: {}\nURL: {}\nScore: {}\nBy: {}\nTime: {}",
            title,
            url,
            story.score.unwrap_or(0),
            story.by.as_deref().unwrap_or("unknown"),
            time
        );

        let p = Paragraph::new(text)
            .style(
                Style::default()
                    .fg(app.theme.foreground)
                    .bg(app.theme.background),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style(Style::default().fg(app.theme.border))
                    .title("Story Details")
                    .title_style(Style::default().fg(app.theme.foreground)),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(p, chunks[0]);

        let comment_area_width = chunks[1].width.saturating_sub(4).max(20) as usize; // Ensure minimum width

        let mut all_lines: Vec<Line> = Vec::new();
        for c in &app.comments {
            let author = c.by.as_deref().unwrap_or("unknown");
            let text = c.text.as_deref().unwrap_or("[deleted]");
            let clean_text = crate::utils::html::extract_text_from_html(text);
            let time = c
                .time
                .as_ref()
                .map(crate::utils::datetime::format_timestamp)
                .unwrap_or_else(|| "unknown".to_string());

            // Author and time line
            all_lines.push(Line::from(vec![
                Span::styled(author, Style::default().fg(app.theme.comment_author)),
                Span::styled(
                    format!(" ({})", time),
                    Style::default().fg(app.theme.comment_time),
                ),
            ]));

            // Wrapped text lines
            let wrapped_text = textwrap::wrap(&clean_text, comment_area_width);
            for line in wrapped_text {
                all_lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(app.theme.foreground),
                )));
            }

            // Separator
            all_lines.push(Line::from(Span::styled(
                "---",
                Style::default().fg(app.theme.border),
            )));
            all_lines.push(Line::from("")); // Empty line for spacing
        }

        let comments_title = if app.comment_ids.is_empty() {
            "Comments (Tab to view Article)".to_string()
        } else {
            format!(
                "Comments ({}/{}) - n: Load More | Tab: Article",
                app.loaded_comments_count,
                app.comment_ids.len()
            )
        };

        let paragraph = Paragraph::new(all_lines)
            .style(Style::default().bg(app.theme.background))
            .scroll((app.comments_scroll as u16, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style(Style::default().fg(app.theme.border))
                    .title(comments_title)
                    .title_style(Style::default().fg(app.theme.foreground)),
            );
        f.render_widget(paragraph, chunks[1]);
    }
}

fn render_article(app: &App, f: &mut Frame, area: Rect) {
    // If we have a selected story, show the same metadata block as in the detail view
    if let Some(story) = &app.selected_story {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        let title = story.title.as_deref().unwrap_or("No Title");
        let url = story.url.as_deref().unwrap_or("No URL");
        let time = story
            .time
            .as_ref()
            .map(crate::utils::datetime::format_timestamp)
            .unwrap_or_else(|| "unknown".to_string());
        let meta_text = format!(
            "Title: {}\nURL: {}\nScore: {}\nBy: {}\nTime: {}",
            title,
            url,
            story.score.unwrap_or(0),
            story.by.as_deref().unwrap_or("unknown"),
            time
        );

        let meta_p = Paragraph::new(meta_text)
            .style(
                Style::default()
                    .fg(app.theme.foreground)
                    .bg(app.theme.background),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style(Style::default().fg(app.theme.border))
                    .title("Story Details")
                    .title_style(Style::default().fg(app.theme.foreground)),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(meta_p, chunks[0]);

        let content = if app.article_loading {
            "Loading article...".to_string()
        } else {
            app.article_content
                .clone()
                .unwrap_or_else(|| "No content available or failed to load.".to_string())
        };

        let p = Paragraph::new(content)
            .style(
                Style::default()
                    .fg(app.theme.foreground)
                    .bg(app.theme.background),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style(Style::default().fg(app.theme.border))
                    .title("Article View (Tab to view Comments)")
                    .title_style(Style::default().fg(app.theme.foreground)),
            )
            .wrap(Wrap { trim: true })
            .scroll((app.article_scroll as u16, 0));
        f.render_widget(p, chunks[1]);
    } else {
        // Fallback: no selected story, render the article content as before
        let content = if app.article_loading {
            "Loading article...".to_string()
        } else {
            app.article_content
                .clone()
                .unwrap_or_else(|| "No content available or failed to load.".to_string())
        };

        let p = Paragraph::new(content)
            .style(
                Style::default()
                    .fg(app.theme.foreground)
                    .bg(app.theme.background),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style(Style::default().fg(app.theme.border))
                    .title("Article View (Tab to view Comments)")
                    .title_style(Style::default().fg(app.theme.foreground)),
            )
            .wrap(Wrap { trim: true })
            .scroll((app.article_scroll as u16, 0));
        f.render_widget(p, area);
    }
}

fn render_top_bar(app: &App, f: &mut Frame, area: Rect) {
    let theme_name = if !app.available_themes.is_empty() {
        let (path, mode) = &app.available_themes[app.current_theme_index];
        let filename = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        format!("Theme: {} ({})", filename, mode)
    } else {
        String::new()
    };

    // Show theme and auto-switch status in the top-right corner
    let auto_status = if app.config.auto_switch_dark_to_light {
        "Auto:On"
    } else {
        "Auto:Off"
    };
    let top_bar_text = format!("{}  {}", theme_name, auto_status);

    let p = Paragraph::new(top_bar_text)
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .padding(Padding::horizontal(1))
                .style(Style::default().bg(app.theme.background)),
        )
        .style(Style::default().fg(app.theme.foreground));
    f.render_widget(p, area);
}

fn render_status_bar(app: &App, f: &mut Frame, area: Rect) {
    let status = if app.loading || app.comments_loading || app.article_loading {
        // Show animated spinner with loading description
        let spinner = app.get_spinner_char();
        let desc = app
            .loading_description()
            .unwrap_or_else(|| "Loading...".to_string());

        // If we have story count info, append it
        if !app.story_ids.is_empty() && app.view_mode == ViewMode::List {
            format!(
                "{} {} | {}/{}",
                spinner,
                desc,
                app.loaded_count,
                app.story_ids.len()
            )
        } else {
            format!("{} {}", spinner, desc)
        }
    } else if app.input_mode == InputMode::Search {
        // Simplified status bar for search mode
        "Search: Type to filter | Enter/Esc: Finish | Ctrl+C: Clear".to_string()
    } else {
        match app.view_mode {
            ViewMode::List => {
                let loaded_info = if !app.story_ids.is_empty() {
                    format!(" | {}/{}", app.loaded_count, app.story_ids.len())
                } else {
                    String::new()
                };
                let filter_hint = if !app.search_query.is_empty() {
                    format!(" | Filter: {}", app.search_query)
                } else {
                    String::new()
                };
                let clear_hint = if !app.search_query.is_empty() {
                    " | C: Clear"
                } else {
                    ""
                };
                format!(
                    "1-6: Cat | /: Search | j/k: Nav | m: More | A: All | Enter: View | t: Theme | ?: Help | q: Quit{}{}{}",
                    loaded_info, filter_hint, clear_hint
                )
            }
            ViewMode::StoryDetail => {
                "Esc/q: Back | o: Browser | n: More Comments | Tab: Article | t: Theme | ?: Help"
                    .to_string()
            }
            ViewMode::Article => {
                "Esc/q: Back | o: Browser | Tab: Comments | j/k: Scroll | t: Theme | ?: Help"
                    .to_string()
            }
        }
    };

    let p = Paragraph::new(status)
        .block(
            Block::default()
                .padding(Padding::horizontal(1))
                .style(Style::default().bg(app.theme.selection_bg)),
        )
        .style(Style::default().fg(app.theme.selection_fg));
    f.render_widget(p, area);
}

fn render_notification(app: &App, f: &mut Frame) {
    if let Some(msg) = &app.notification_message {
        let area = f.area();

        // Create centered popup
        let popup_width = (msg.len() as u16 + 4).min(area.width - 4);
        let popup_height = 3;

        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Clear background
        let popup = Paragraph::new(msg.as_str())
            .style(
                Style::default()
                    .bg(app.theme.selection_bg)
                    .fg(app.theme.selection_fg)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.border))
                    .title("Info")
                    .title_style(Style::default().fg(app.theme.foreground)),
            )
            .alignment(Alignment::Center);

        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }
}

fn render_search_overlay(app: &App, f: &mut Frame) {
    let area = f.area();

    // Create search box at the top center
    let search_width = 60.min(area.width - 4);
    let search_height = 3;

    let search_x = (area.width.saturating_sub(search_width)) / 2;
    let search_y = (area.height.saturating_sub(search_height)) / 2; // Centered vertically

    let search_area = Rect::new(search_x, search_y, search_width, search_height);

    // Display the search query with cursor
    let display_text = format!("{}█", app.search_query); // █ as cursor

    let search_box = Paragraph::new(display_text)
        .style(
            Style::default()
                .fg(app.theme.foreground)
                .bg(app.theme.background),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.selection_bg))
                .title(" Search (Esc to cancel) ")
                .title_style(
                    Style::default()
                        .fg(app.theme.selection_fg)
                        .bg(app.theme.selection_bg)
                        .add_modifier(Modifier::BOLD),
                ),
        );

    f.render_widget(Clear, search_area);
    f.render_widget(search_box, search_area);
}

fn render_help_overlay(app: &App, f: &mut Frame) {
    let area = f.area();

    // Create centered popup
    let popup_width = 80.min(area.width - 4);
    let popup_height = 20.min(area.height - 4);

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear background
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.selection_bg))
        .title(" Keyboard Shortcuts (Esc/q to close) ")
        .title_style(
            Style::default()
                .fg(app.theme.selection_fg)
                .bg(app.theme.selection_bg)
                .add_modifier(Modifier::BOLD),
        )
        .padding(Padding::horizontal(2))
        .style(Style::default().bg(app.theme.background));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Shortcuts content
    let shortcuts = vec![
        Line::from(vec![Span::styled(
            "Global",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(app.theme.selection_bg),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("?", Style::default().fg(app.theme.comment_time)),
            Span::raw("        Show this help"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("q / Esc", Style::default().fg(app.theme.comment_time)),
            Span::raw("  Quit / Back / Close overlay"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("t", Style::default().fg(app.theme.comment_time)),
            Span::raw("        Toggle theme (Light/Dark)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("g", Style::default().fg(app.theme.comment_time)),
            Span::raw("        Toggle auto-switch theme"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(app.theme.selection_bg),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("j / k", Style::default().fg(app.theme.comment_time)),
            Span::raw("    Move selection down / up"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Enter", Style::default().fg(app.theme.comment_time)),
            Span::raw("    View story details"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Tab", Style::default().fg(app.theme.comment_time)),
            Span::raw("      Toggle Article/Comments view"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Story List",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(app.theme.selection_bg),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("1-6", Style::default().fg(app.theme.comment_time)),
            Span::raw("      Switch categories (Top, New, Best...)"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("/", Style::default().fg(app.theme.comment_time)),
            Span::raw("        Search stories"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("m", Style::default().fg(app.theme.comment_time)),
            Span::raw("        Load more stories"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("A", Style::default().fg(app.theme.comment_time)),
            Span::raw("        Load ALL stories"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Article / Comments",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(app.theme.selection_bg),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("o", Style::default().fg(app.theme.comment_time)),
            Span::raw("        Open in browser"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("n", Style::default().fg(app.theme.comment_time)),
            Span::raw("        Load more comments"),
        ]),
    ];

    let p = Paragraph::new(shortcuts)
        .style(Style::default().fg(app.theme.foreground))
        .wrap(Wrap { trim: false }); // Don't trim to preserve indentation

    f.render_widget(p, inner_area);
}

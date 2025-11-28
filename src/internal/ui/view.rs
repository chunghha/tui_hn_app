#![allow(clippy::single_match)]
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
        ViewMode::Bookmarks => render_list(app, f, chunks[1]),
    }

    render_status_bar(app, f, chunks[2]);

    // Render search overlay if in search mode
    match app.input_mode {
        InputMode::Search => render_search_overlay(app, f),
        _ => {}
    }

    // Render notification overlay if present
    match app.notification_message {
        Some(_) => render_notification(app, f),
        None => {}
    }

    // Render progress overlay if loading all stories
    match app.story_load_progress {
        Some(_) => render_progress_overlay(app, f),
        None => {}
    }

    // Render help overlay if active
    match app.show_help {
        true => render_help_overlay(app, f),
        false => {}
    }
}

fn render_progress_overlay(app: &App, f: &mut Frame) {
    match app.story_load_progress {
        Some((loaded, total)) => {
            let area = f.area();
            let popup_width = 60.min(area.width - 4);
            let popup_height = 5; // Reduced height
            let popup_x = (area.width.saturating_sub(popup_width)) / 2;
            let popup_y = (area.height.saturating_sub(popup_height)) / 2;
            let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

            // Add spinner to title
            let spinner = app.get_spinner_char();
            let percent = match total {
                0 => 0,
                t => (loaded as f64 / t as f64 * 100.0) as u16,
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
        None => {}
    }
}

fn render_list(app: &mut App, f: &mut Frame, area: Rect) {
    // Determine which stories to display based on view mode
    let stories_to_display: Vec<_> = match app.view_mode {
        ViewMode::Bookmarks => {
            // Convert bookmarked stories to Story objects for display
            app.bookmarks
                .stories
                .iter()
                .enumerate()
                .filter_map(|(idx, bookmarked)| {
                    // Find the full story in app.stories if available
                    app.stories
                        .iter()
                        .find(|s| s.id == bookmarked.id)
                        .map(|story| (idx, story))
                })
                .collect()
        }
        _ => {
            // Filter stories based on search query for normal list view
            match app.search_query.as_str() {
                "" => app.stories.iter().enumerate().collect(),
                query_str => {
                    let query = query_str.to_lowercase();
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
                }
            }
        }
    };

    let items: Vec<ListItem> = stories_to_display
        .iter()
        .map(|(_, story)| {
            let title = story.title.as_deref().unwrap_or("No Title");
            let score = story.score.unwrap_or(0);
            let by = story.by.as_deref().unwrap_or("unknown");
            let comments = story.descendants.unwrap_or(0);

            // Extract domain from URL
            let domain = story
                .url
                .as_ref()
                .and_then(|url| crate::utils::url::extract_domain(url))
                .map(|d| format!(" ({})", d))
                .unwrap_or_default();

            let time = story
                .time
                .as_ref()
                .map(crate::utils::datetime::format_timestamp)
                .unwrap_or_else(|| "unknown".to_string());

            // Add bookmark indicator if story is bookmarked
            let bookmark_indicator = match app.bookmarks.contains(story.id) {
                true => "★ ",
                false => "",
            };

            // Title line with score, bookmark indicator, and domain
            let title_line = Line::from(vec![
                Span::styled(format!("{} ", score), Style::default().fg(app.theme.score)),
                Span::styled(
                    bookmark_indicator,
                    Style::default().fg(app.theme.selection_bg),
                ),
                Span::styled(title, Style::default().fg(app.theme.foreground)),
                Span::styled(domain, Style::default().fg(app.theme.comment_time)),
            ]);

            // Metadata line with age, comments, and author
            let meta_line = Line::from(vec![
                Span::styled("    ", Style::default()), // Indent
                Span::styled(time, Style::default().fg(app.theme.comment_time)),
                Span::styled(" | ", Style::default().fg(app.theme.border)),
                Span::styled(
                    format!("{} comments", comments),
                    Style::default().fg(app.theme.comment_time),
                ),
                Span::styled(" | by ", Style::default().fg(app.theme.border)),
                Span::styled(by, Style::default().fg(app.theme.comment_author)),
            ]);

            ListItem::new(vec![title_line, meta_line])
        })
        .collect();

    // Place the version next to the "Hacker News" label in the title
    let title = match app.search_query.as_str() {
        "" => format!(
            "Hacker News v{} - {}",
            app.app_version, app.current_list_type
        ),
        query => format!(
            "Hacker News v{} - {} (Filter: {})",
            app.app_version, app.current_list_type, query
        ),
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
        let mut skip_until_depth: Option<usize> = None;

        for row in &app.comments {
            // Skip collapsed children
            if let Some(until_depth) = skip_until_depth {
                match row.depth.cmp(&until_depth) {
                    std::cmp::Ordering::Greater => continue,
                    _ => skip_until_depth = None,
                }
            }

            let author = row.comment.by.as_deref().unwrap_or("unknown");
            let text = row.comment.text.as_deref().unwrap_or("[deleted]");
            let clean_text = crate::utils::html::extract_text_from_html(text);
            let time = row
                .comment
                .time
                .as_ref()
                .map(crate::utils::datetime::format_timestamp)
                .unwrap_or_else(|| "unknown".to_string());

            // Indentation and visual guides
            let indent = "  ".repeat(row.depth);
            let mut guide = String::new();
            for i in 0..row.depth {
                match i.cmp(&row.depth.saturating_sub(1)) {
                    std::cmp::Ordering::Less => guide.push_str("│ "),
                    _ => guide.push_str("└─"),
                }
            }

            // Collapse indicator
            let has_kids =
                row.comment.kids.is_some() && !row.comment.kids.as_ref().unwrap().is_empty();
            let collapse_indicator = match (has_kids, row.expanded) {
                (true, true) => "[-] ",
                (true, false) => "[+] ",
                _ => "",
            };

            // Set skip flag if collapsed
            if has_kids && !row.expanded {
                skip_until_depth = Some(row.depth);
            }

            // Author and time line with indentation
            all_lines.push(Line::from(vec![
                Span::styled(guide.clone(), Style::default().fg(app.theme.border)),
                Span::styled(
                    collapse_indicator,
                    Style::default().fg(app.theme.comment_time),
                ),
                Span::styled(author, Style::default().fg(app.theme.comment_author)),
                Span::styled(
                    format!(" ({})", time),
                    Style::default().fg(app.theme.comment_time),
                ),
            ]));

            // Wrapped text lines with indentation
            let available_width = comment_area_width.saturating_sub(row.depth * 2);
            let wrapped_text = textwrap::wrap(&clean_text, available_width.max(20));
            for line in wrapped_text {
                all_lines.push(Line::from(vec![
                    Span::styled(indent.clone(), Style::default()),
                    Span::styled(line.to_string(), Style::default().fg(app.theme.foreground)),
                ]));
            }

            // Separator
            all_lines.push(Line::from(Span::styled(
                "---",
                Style::default().fg(app.theme.border),
            )));
            all_lines.push(Line::from("")); // Empty line for spacing
        }

        let comments_title = match app.comment_ids.len() {
            0 => "Comments (Tab to view Article)".to_string(),
            len => format!(
                "Comments ({}/{}) - n: Load More | Tab: Article",
                app.loaded_comments_count, len
            ),
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
    match &app.selected_story {
        Some(story) => {
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

            let content_lines = match (app.article_loading, &app.article_content) {
                (true, _) => vec![Line::from("Loading article...")],
                (false, Some(article)) => {
                    let mut lines = Vec::new();
                    if !article.title.is_empty() {
                        lines.push(Line::from(Span::styled(
                            &article.title,
                            Style::default()
                                .fg(app.theme.foreground)
                                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        )));
                        lines.push(Line::from(""));
                    }

                    for element in &article.elements {
                        match element {
                            crate::internal::models::ArticleElement::Paragraph(text) => {
                                lines.push(Line::from(Span::styled(
                                    text,
                                    Style::default().fg(app.theme.foreground),
                                )));
                                lines.push(Line::from(""));
                            }
                            crate::internal::models::ArticleElement::Heading(level, text) => {
                                let style = match level {
                                    1 => Style::default()
                                        .fg(app.theme.foreground)
                                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                                    2 => Style::default()
                                        .fg(app.theme.foreground)
                                        .add_modifier(Modifier::BOLD),
                                    _ => Style::default()
                                        .fg(app.theme.foreground)
                                        .add_modifier(Modifier::ITALIC),
                                };
                                lines.push(Line::from(Span::styled(text, style)));
                                lines.push(Line::from(""));
                            }
                            crate::internal::models::ArticleElement::CodeBlock { lang, code } => {
                                let lang_info = lang.as_deref().unwrap_or("text");
                                lines.push(Line::from(Span::styled(
                                    format!("```{}", lang_info),
                                    Style::default().fg(app.theme.comment_time),
                                )));
                                for line in code.lines() {
                                    lines.push(Line::from(Span::styled(
                                        line,
                                        Style::default().fg(app.theme.comment_author), // Use a different color for code
                                    )));
                                }
                                lines.push(Line::from(Span::styled(
                                    "```",
                                    Style::default().fg(app.theme.comment_time),
                                )));
                                lines.push(Line::from(""));
                            }
                            crate::internal::models::ArticleElement::List(items) => {
                                for item in items {
                                    lines.push(Line::from(vec![
                                        Span::styled(" • ", Style::default().fg(app.theme.border)),
                                        Span::styled(
                                            item,
                                            Style::default().fg(app.theme.foreground),
                                        ),
                                    ]));
                                }
                                lines.push(Line::from(""));
                            }
                            crate::internal::models::ArticleElement::Table(rows) => {
                                lines.push(Line::from(Span::styled(
                                    "[Table]",
                                    Style::default()
                                        .fg(app.theme.comment_time)
                                        .add_modifier(Modifier::ITALIC),
                                )));
                                // Simple ASCII rendering for now
                                for row in rows {
                                    let row_text = row.join(" | ");
                                    lines.push(Line::from(Span::styled(
                                        format!("| {} |", row_text),
                                        Style::default().fg(app.theme.foreground),
                                    )));
                                }
                                lines.push(Line::from(""));
                            }
                            crate::internal::models::ArticleElement::Image(alt) => {
                                lines.push(Line::from(Span::styled(
                                    format!("[IMAGE: {}]", alt),
                                    Style::default()
                                        .fg(app.theme.comment_time)
                                        .add_modifier(Modifier::ITALIC),
                                )));
                                lines.push(Line::from(""));
                            }
                            crate::internal::models::ArticleElement::Quote(text) => {
                                lines.push(Line::from(vec![
                                    Span::styled("│ ", Style::default().fg(app.theme.border)),
                                    Span::styled(
                                        text,
                                        Style::default()
                                            .fg(app.theme.foreground)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ]));
                                lines.push(Line::from(""));
                            }
                        }
                    }
                    lines
                }
                (false, None) => vec![Line::from("No content available or failed to load.")],
            };

            let p = Paragraph::new(content_lines)
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
        }
        None => {
            // Fallback: no selected story, render empty or loading
            let content_lines = match (app.article_loading, &app.article_content) {
                (true, _) => vec![Line::from("Loading article...")],
                (false, Some(_)) => vec![Line::from("Select a story to view article.")],
                (false, None) => vec![Line::from("Select a story to view article.")],
            };

            let p = Paragraph::new(content_lines)
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
}

fn render_top_bar(app: &App, f: &mut Frame, area: Rect) {
    let theme_name = match app.available_themes.get(app.current_theme_index) {
        Some((path, mode)) => {
            let filename = Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            format!("Theme: {} ({})", filename, mode)
        }
        None => String::new(),
    };

    // Show theme and auto-switch status in the top-right corner
    let auto_status = match app.config.auto_switch_dark_to_light {
        true => "Auto:On",
        false => "Auto:Off",
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
    let status = match (
        app.loading || app.comments_loading || app.article_loading,
        &app.input_mode,
        &app.view_mode,
    ) {
        (true, _, &ViewMode::List) if !app.story_ids.is_empty() => {
            // Show animated spinner with loading description and story counts
            let spinner = app.get_spinner_char();
            let desc = app
                .loading_description()
                .unwrap_or_else(|| "Loading...".to_string());
            format!(
                "{} {} | {}/{}",
                spinner,
                desc,
                app.loaded_count,
                app.story_ids.len()
            )
        }
        (true, _, _) => {
            // Show animated spinner with loading description
            let spinner = app.get_spinner_char();
            let desc = app
                .loading_description()
                .unwrap_or_else(|| "Loading...".to_string());
            format!("{} {}", spinner, desc)
        }
        (false, &InputMode::Search, _) => {
            // Simplified status bar for search mode
            "Search: Type to filter | Enter/Esc: Finish | Ctrl+C: Clear".to_string()
        }
        (false, _, &ViewMode::List) => {
            let loaded_info = match app.story_ids.len() {
                0 => String::new(),
                len => format!(" | {}/{}", app.loaded_count, len),
            };
            let filter_hint = match app.search_query.as_str() {
                "" => String::new(),
                q => format!(" | Filter: {}", q),
            };
            let clear_hint = match app.search_query.is_empty() {
                false => " | C: Clear",
                true => "",
            };
            format!(
                "1-6: Cat | /: Search | j/k: Nav | m: More | A: All | Enter: View | b: Bookmark | B: View Bookmarks | t: Theme | ?: Help | q: Quit{}{}{}",
                loaded_info, filter_hint, clear_hint
            )
        }
        (false, _, &ViewMode::StoryDetail) => {
            "Esc/q: Back | o: Browser | b: Bookmark | n: More Comments | Tab: Article | t: Theme | ?: Help"
                .to_string()
        }
        (false, _, &ViewMode::Article) => {
            "Esc/q: Back | o: Browser | Tab: Comments | j/k: Scroll | t: Theme | ?: Help"
                .to_string()
        }
        (false, _, &ViewMode::Bookmarks) => {
            // Show a compact status for bookmarks view, including count
            let count = app.bookmarks.stories.len();
            let bookmark_info = match count {
                0 => "No bookmarks".to_string(),
                n => format!("Bookmarks: {}", n),
            };
            format!(
                "Esc/q: Back | Enter: View | t: Theme | ?: Help | {}",
                bookmark_info
            )
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
    match &app.notification_message {
        Some(msg) => {
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
        None => {}
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
    let popup_width = 56.min(area.width - 4);
    let popup_height = 26.min(area.height - 4);

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
            "Bookmarks",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(app.theme.selection_bg),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("b", Style::default().fg(app.theme.comment_time)),
            Span::raw("        Toggle bookmark on selected story"),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("B", Style::default().fg(app.theme.comment_time)),
            Span::raw("        View bookmarked stories"),
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

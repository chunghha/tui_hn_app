#![allow(clippy::single_match)]
use std::path::Path;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
};
use textwrap;

use super::app::{App, InputMode, ViewMode};
use super::sort::{SortBy, SortOrder};

#[tracing::instrument(skip(app, f))]
pub fn draw(app: &mut App, f: &mut Frame) {
    // High level render timing. This is conditionalally logged at the end of draw
    // when performance metrics are enabled and in debug builds.
    let start = std::time::Instant::now();

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
        ViewMode::List => {
            let view_start = std::time::Instant::now();
            render_list(app, f, chunks[1]);
            if app.config.logging.enable_performance_metrics && cfg!(debug_assertions) {
                tracing::debug!(elapsed = ?view_start.elapsed(), view = "list", "render.list");
            }
        }
        ViewMode::StoryDetail => {
            let view_start = std::time::Instant::now();
            render_detail(app, f, chunks[1]);
            if app.config.logging.enable_performance_metrics && cfg!(debug_assertions) {
                tracing::debug!(elapsed = ?view_start.elapsed(), view = "detail", "render.detail");
            }
        }
        ViewMode::Article => {
            let view_start = std::time::Instant::now();
            render_article(app, f, chunks[1]);
            if app.config.logging.enable_performance_metrics && cfg!(debug_assertions) {
                tracing::debug!(elapsed = ?view_start.elapsed(), view = "article", "render.article");
            }
        }
        ViewMode::Bookmarks => {
            let view_start = std::time::Instant::now();
            render_list(app, f, chunks[1]);
            if app.config.logging.enable_performance_metrics && cfg!(debug_assertions) {
                tracing::debug!(elapsed = ?view_start.elapsed(), view = "bookmarks", "render.bookmarks");
            }
        }
        ViewMode::History => {
            let view_start = std::time::Instant::now();
            render_list(app, f, chunks[1]);
            if app.config.logging.enable_performance_metrics && cfg!(debug_assertions) {
                tracing::debug!(elapsed = ?view_start.elapsed(), view = "history", "render.history");
            }
        }
    }

    render_status_bar(app, f, chunks[2]);

    // Render search overlay if in search mode
    match app.input_mode {
        InputMode::Search => render_search_overlay(app, f),
        _ => {}
    }

    // Render notification overlay if present
    if app.notification.is_some() {
        render_notification(app, f);
    }

    // Render progress overlay if loading all stories
    match app.story_load_progress {
        Some(_) => render_progress_overlay(app, f),
        None => {}
    }

    // Render help overlay if active
    if app.show_help {
        render_help_overlay(app, f);
    }

    // Render theme editor overlay if active
    if app.theme_editor.active {
        render_theme_editor_overlay(app, f);
    }

    // Render log viewer overlay if active
    if app.log_viewer.visible {
        app.log_viewer.render(f, f.area());
    }

    // Conditional render timing: only emit when the config allows it and during debug builds
    if app.config.logging.enable_performance_metrics && cfg!(debug_assertions) {
        tracing::debug!(elapsed = ?start.elapsed(), "render.draw");
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
                        .map(|story| (idx, story.clone()))
                })
                .collect()
        }
        ViewMode::History => {
            app.history
                .stories
                .iter()
                .enumerate()
                .map(|(idx, viewed)| {
                    let story = crate::internal::models::Story {
                        id: viewed.id,
                        title: Some(viewed.title.clone()),
                        url: viewed.url.clone(),
                        by: viewed.by.clone(),
                        score: viewed.score,
                        // Use viewed_at as the time so it shows "viewed X ago"
                        time: Some(viewed.viewed_at.timestamp().as_second()),
                        descendants: viewed.descendants,
                        kids: None,
                    };
                    (idx, story)
                })
                .collect()
        }
        _ => {
            // Filter stories based on search query for normal list view
            match app.search_query.is_empty() {
                true => app
                    .stories
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (i, s.clone()))
                    .collect(),
                false => {
                    use crate::internal::search::SearchMode;
                    app.stories
                        .iter()
                        .enumerate()
                        .filter(|(_, story)| {
                            let title_match = story
                                .title
                                .as_ref()
                                .map(|t| app.search_query.matches(t))
                                .unwrap_or(false);

                            match app.search_query.mode {
                                SearchMode::Title => title_match,
                                SearchMode::Comments => {
                                    // Search in cached comments (if available)
                                    // For now, we don't have easy access to comments here
                                    // This would require pre-fetching comments or maintaining a comment cache
                                    // For v0.5.2, we'll just return false for comment-only search in list view
                                    false
                                }
                                SearchMode::TitleAndComments => {
                                    // In list view, we can only effectively search titles
                                    title_match
                                }
                            }
                        })
                        .map(|(i, s)| (i, s.clone()))
                        .collect()
                }
            }
        }
    };

    let items: Vec<ListItem> = stories_to_display
        .iter()
        .enumerate()
        .map(|(idx, (_, story))| {
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

            // Show score with leading space for proper alignment
            let score = format!("{:3} ", score);

            // Check if story is bookmarked
            let bookmark_indicator = match app.bookmarks.contains(story.id) {
                true => "★ ",
                false => "",
            };

            // Build title line with optional fields based on config
            let mut title_spans = vec![Span::styled(
                format!("{:<4}", idx + 1),
                Style::default().fg(app.theme.comment_time),
            )];

            // Add bookmark indicator
            title_spans.push(Span::styled(
                bookmark_indicator,
                Style::default().fg(app.theme.selection_bg),
            ));

            // Add score if configured
            if app.config.ui.list_view.show_score {
                title_spans.push(Span::styled(
                    format!("{} ", score),
                    Style::default().fg(app.theme.score),
                ));
            }

            title_spans.push(Span::styled(
                title,
                Style::default().fg(app.theme.foreground),
            ));

            // Add domain if configured
            if app.config.ui.list_view.show_domain {
                title_spans.push(Span::styled(
                    domain,
                    Style::default().fg(app.theme.comment_time),
                ));
            }

            let title_line = Line::from(title_spans);

            // Build metadata line with optional fields
            let mut meta_spans = vec![Span::styled("    ", Style::default())]; // Indent
            let mut first_field = true;

            // Add time if configured
            if app.config.ui.list_view.show_age {
                meta_spans.push(Span::styled(
                    time,
                    Style::default().fg(app.theme.comment_time),
                ));
                first_field = false;
            }

            // Add comments if configured
            if app.config.ui.list_view.show_comments {
                if !first_field {
                    meta_spans.push(Span::styled(" | ", Style::default().fg(app.theme.border)));
                }
                meta_spans.push(Span::styled(
                    format!("{} comments", comments),
                    Style::default().fg(app.theme.comment_time),
                ));
                first_field = false;
            }

            // Always show author
            match first_field {
                false => meta_spans.push(Span::styled(
                    " | by ",
                    Style::default().fg(app.theme.border),
                )),
                true => meta_spans.push(Span::styled("by ", Style::default().fg(app.theme.border))),
            }
            meta_spans.push(Span::styled(
                by,
                Style::default().fg(app.theme.comment_author),
            ));

            let meta_line = Line::from(meta_spans);

            ListItem::new(vec![title_line, meta_line])
        })
        .collect();

    // Place the version next to the "Hacker News" label in the title
    let sort_indicator = format!(
        " (sorted by {} {})",
        match app.sort_by {
            SortBy::Score => "Score",
            SortBy::Comments => "Comments",
            SortBy::Time => "Time",
        },
        match app.sort_order {
            SortOrder::Ascending => "asc",
            SortOrder::Descending => "desc",
        }
    );

    let title = match app.search_query.is_empty() {
        true => format!(
            "Hacker News v{} - {}{}",
            app.app_version, app.current_list_type, sort_indicator
        ),
        false => format!(
            "Hacker News v{} - {} (Filter: {} [{}|{}])",
            app.app_version,
            app.current_list_type,
            app.search_query.query,
            app.search_query.mode.as_str(),
            app.search_query.search_type.as_str()
        ),
    };

    let title = match app.view_mode {
        ViewMode::History => format!("History ({} stories)", items.len()),
        ViewMode::Bookmarks => format!("Bookmarks ({} stories)", items.len()),
        _ => title,
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .padding(Padding::new(
                    app.config.ui.padding.horizontal,
                    app.config.ui.padding.horizontal,
                    app.config.ui.padding.vertical,
                    app.config.ui.padding.vertical,
                ))
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
                    .padding(Padding::new(
                        app.config.ui.padding.horizontal,
                        app.config.ui.padding.horizontal,
                        app.config.ui.padding.vertical,
                        app.config.ui.padding.vertical,
                    ))
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
                    .padding(Padding::new(
                        app.config.ui.padding.horizontal,
                        app.config.ui.padding.horizontal,
                        app.config.ui.padding.vertical,
                        app.config.ui.padding.vertical,
                    ))
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
                        .padding(Padding::new(
                            app.config.ui.padding.horizontal,
                            app.config.ui.padding.horizontal,
                            app.config.ui.padding.vertical,
                            app.config.ui.padding.vertical,
                        ))
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
                        .padding(Padding::new(
                            app.config.ui.padding.horizontal,
                            app.config.ui.padding.horizontal,
                            app.config.ui.padding.vertical,
                            app.config.ui.padding.vertical,
                        ))
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
                        .padding(Padding::new(
                            app.config.ui.padding.horizontal,
                            app.config.ui.padding.horizontal,
                            app.config.ui.padding.vertical,
                            app.config.ui.padding.vertical,
                        ))
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
                .padding(Padding::new(
                    app.config.ui.padding.horizontal,
                    app.config.ui.padding.horizontal,
                    app.config.ui.padding.vertical,
                    app.config.ui.padding.vertical,
                ))
                .style(Style::default().bg(app.theme.background)),
        )
        .style(Style::default().fg(app.theme.foreground));
    f.render_widget(p, area);
}

/// Parse status bar format tokens and replace with actual values
fn parse_status_bar_format(app: &App, format: &str) -> String {
    let mut result = format.to_string();

    // {mode} - Current view mode
    let mode_str = match app.view_mode {
        ViewMode::List => "List",
        ViewMode::StoryDetail => "Story",
        ViewMode::Article => "Article",
        ViewMode::Bookmarks => "Bookmarks",
        ViewMode::History => "History",
    };
    result = result.replace("{mode}", mode_str);

    // {category} - Story category
    let category_str = match app.current_list_type {
        crate::api::StoryListType::Top => "Top",
        crate::api::StoryListType::New => "New",
        crate::api::StoryListType::Best => "Best",
        crate::api::StoryListType::Ask => "Ask",
        crate::api::StoryListType::Show => "Show",
        crate::api::StoryListType::Job => "Job",
    };
    result = result.replace("{category}", category_str);

    // {count} - Loaded story count
    result = result.replace("{count}", &app.loaded_count.to_string());

    // {total} - Total story count
    result = result.replace("{total}", &app.story_ids.len().to_string());

    // {sort} - Sort field
    let sort_str = match app.sort_by {
        crate::internal::ui::sort::SortBy::Score => "Score",
        crate::internal::ui::sort::SortBy::Comments => "Comments",
        crate::internal::ui::sort::SortBy::Time => "Time",
    };
    result = result.replace("{sort}", sort_str);

    // {order} - Sort order
    let order_str = match app.sort_order {
        crate::internal::ui::sort::SortOrder::Ascending => "↑",
        crate::internal::ui::sort::SortOrder::Descending => "↓",
    };
    result = result.replace("{order}", order_str);

    // {search} - Search query
    let search_str = app.search_query.query.as_str();
    result = result.replace("{search}", search_str);

    // {spinner} - Loading spinner
    let spinner_str = match (app.loading, app.comments_loading, app.article_loading) {
        (true, _, _) | (_, true, _) | (_, _, true) => app.get_spinner_char().to_string(),
        _ => String::new(),
    };
    result = result.replace("{spinner}", &spinner_str);

    // {loading_text} - Loading description
    let loading_text = app.loading_description().unwrap_or_default();
    result = result.replace("{loading_text}", &loading_text);

    // {theme} - Current theme name
    let theme_name = app
        .available_themes
        .get(app.current_theme_index)
        .map(|(filename, _)| {
            std::path::Path::new(filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
        })
        .unwrap_or("Default");
    result = result.replace("{theme}", theme_name);

    // {shortcuts} - Context-sensitive shortcuts (fallback to default behavior)
    let shortcuts = match app.view_mode {
        ViewMode::List => "j/k:Nav | Enter:View | b:Bookmark | ?:Help | L:Log | q:Quit",
        ViewMode::StoryDetail => "Esc:Back | o:Browser | Tab:Article | ?:Help",
        ViewMode::Article => "Esc:Back | j/k:Scroll | Tab:Comments | ?:Help",
        ViewMode::Bookmarks => "Enter:View | Esc:Back | ?:Help",
        ViewMode::History => "Enter:View | X:Clear | Esc:Back | ?:Help",
    };
    result = result.replace("{shortcuts}", shortcuts);

    result
}

fn get_verbose_status(app: &App) -> String {
    if app.loading || app.comments_loading || app.article_loading {
        let desc = app
            .loading_description()
            .unwrap_or_else(|| "content".to_string());
        return format!("Loading {}. Please wait.", desc);
    }

    match &app.view_mode {
        ViewMode::List => {
            let list_type = app.current_list_type.to_string();
            format!(
                "Viewing {} Stories. {} stories loaded. Press Question Mark for help.",
                list_type, app.loaded_count
            )
        }
        ViewMode::StoryDetail => {
            if let Some(story) = app.selected_story.as_ref() {
                format!(
                    "Viewing Story: {}. By {}. {} comments. Press Tab to view article.",
                    story.title.as_deref().unwrap_or("Unknown Title"),
                    story.by.as_deref().unwrap_or("Unknown Author"),
                    story.descendants.unwrap_or(0)
                )
            } else {
                "Viewing Story Detail. No story selected.".to_string()
            }
        }
        ViewMode::Article => {
            if let Some(story) = app.selected_story.as_ref() {
                format!(
                    "Reading Article: {}. Press Escape to return to story details.",
                    story.title.as_deref().unwrap_or("Unknown Title")
                )
            } else {
                "Reading Article. No article loaded.".to_string()
            }
        }
        _ => "Viewing Content.".to_string(),
    }
}

fn render_status_bar(app: &App, f: &mut Frame, area: Rect) {
    // Check if custom format is configured
    let status = match (
        app.config.accessibility.verbose_status,
        app.config.ui.status_bar_format.is_empty(),
        app.loading || app.comments_loading || app.article_loading,
        &app.input_mode,
        &app.view_mode,
    ) {
        (true, _, _, _, _) => get_verbose_status(app),
        (false, false, _, _, _) => {
            // Use custom format with token parsing
            parse_status_bar_format(app, &app.config.ui.status_bar_format)
        }
        (false, true, true, _, &ViewMode::List) if !app.story_ids.is_empty() => {
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
        (false, true, true, _, _) => {
            // Show animated spinner with loading description
            let spinner = app.get_spinner_char();
            let desc = app
                .loading_description()
                .unwrap_or_else(|| "Loading...".to_string());
            format!("{} {}", spinner, desc)
        }
        (false, true, false, &InputMode::Search, _) => {
            // Enhanced status bar for search mode with shortcuts
            "Search: Type | ↑↓: History | Ctrl+M/F2: Mode | Ctrl+R/F3: Regex | Enter: OK | Esc: Cancel".to_string()
        }
        (false, true, false, _, &ViewMode::List) => {
            let loaded_info = match app.story_ids.len() {
                0 => String::new(),
                len => format!(" | {}/{}", app.loaded_count, len),
            };
            let filter_hint = match app.search_query.query.as_str() {
                "" => String::new(),
                q => format!(" | Filter: {}", q),
            };
            let clear_hint = if app.search_query.is_empty() {
                ""
            } else {
                " | Q: Clear"
            };
            format!(
                "1-6: Cat | /: Search | j/k: Nav | m: More | A: All | Enter: View | b: Bookmark | B/H: View B/H | t: Theme | ?: Help | q: Quit{}{}{}",
                loaded_info, filter_hint, clear_hint
            )
        }
        (false, true, false, _, &ViewMode::StoryDetail) => {
            "Esc/q: Back | o: Browser | b: Bookmark | n: More Comments | Tab: Article | t: Theme | ?: Help"
                .to_string()
        }
        (false, true, false, _, &ViewMode::Article) => {
            "Esc/q: Back | o: Browser | Tab: Comments | j/k: Scroll | t: Theme | ?: Help"
                .to_string()
        }
        (false, true, false, _, &ViewMode::Bookmarks) => {
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
        (false, true, false, _, &ViewMode::History) => {
            let count = app.history.stories.len();
            let history_info = match count {
                0 => "No history".to_string(),
                n => format!("History: {}", n),
            };
            format!(
                "Esc/q: Back | Enter: View | X: Clear History | t: Theme | ?: Help | {}",
                history_info
            )
        }
    };

    let p = Paragraph::new(status)
        .block(
            Block::default()
                .padding(Padding::new(
                    app.config.ui.padding.horizontal,
                    app.config.ui.padding.horizontal,
                    app.config.ui.padding.vertical,
                    app.config.ui.padding.vertical,
                ))
                .style(Style::default().bg(app.theme.selection_bg)),
        )
        .style(Style::default().fg(app.theme.selection_fg));
    f.render_widget(p, area);
}

fn render_notification(app: &App, f: &mut Frame) {
    if let Some(notification) = &app.notification {
        let area = f.area();

        // Create centered popup
        let popup_width = (notification.message.len() as u16 + 4).min(area.width - 4);
        let popup_height = 3;

        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Color code based on notification type
        use crate::internal::notification::NotificationType;
        let (bg_color, title) = match notification.notification_type {
            NotificationType::Info => (Color::Blue, "Info"),
            NotificationType::Warning => (Color::Yellow, "Warning"),
            NotificationType::Error => (Color::Red, "Error"),
        };

        let popup = Paragraph::new(notification.message.as_str())
            .style(
                Style::default()
                    .bg(bg_color)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.border))
                    .title(title)
                    .title_style(Style::default().fg(app.theme.foreground)),
            )
            .alignment(Alignment::Center);

        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }
}

fn render_search_overlay(app: &App, f: &mut Frame) {
    let area = f.area();

    // Create search box at the top center - make it taller for more info
    let search_width = 70.min(area.width - 4);
    let search_height = 5;

    let search_x = (area.width.saturating_sub(search_width)) / 2;
    let search_y = (area.height.saturating_sub(search_height)) / 2;

    let search_area = Rect::new(search_x, search_y, search_width, search_height);

    // Build title with mode and type indicators
    let title = format!(
        " Search: {} | {} ",
        app.search_query.mode.as_str(),
        app.search_query.search_type.as_str()
    );

    // Display the temp search input with cursor
    let mut display_lines = vec![];

    // Input line with cursor
    let input_line = format!("{}█", app.temp_search_input);
    display_lines.push(Line::from(Span::styled(
        input_line,
        Style::default().fg(app.theme.foreground),
    )));

    // Show regex error if present
    if let Some(ref error) = app.search_query.regex_error {
        display_lines.push(Line::from(Span::styled(
            error.clone(),
            Style::default().fg(app.theme.score), // Use score color (typically red/orange)
        )));
    }

    let search_box = Paragraph::new(display_lines)
        .style(Style::default().bg(app.theme.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.selection_bg))
                .title(title)
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
    let popup_height = 30.min(area.height - 4);

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
        .padding(Padding::new(
            app.config.ui.padding.horizontal,
            app.config.ui.padding.horizontal,
            app.config.ui.padding.vertical,
            app.config.ui.padding.vertical,
        ))
        .style(Style::default().bg(app.theme.background));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Shortcuts content based on page
    let shortcuts = match app.help_page {
        1 => vec![
            Line::from(vec![Span::styled(
                "General Shortcuts (Tab for Theme Editor)",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(app.theme.selection_bg),
            )]),
            Line::from(""),
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
                Span::styled("S/C/T", Style::default().fg(app.theme.comment_time)),
                Span::raw("    Sort by Score/Comments/Time"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("O", Style::default().fg(app.theme.comment_time)),
                Span::raw("        Toggle sort order (asc/desc)"),
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
                "Bookmarks & History",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(app.theme.selection_bg),
            )]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("b", Style::default().fg(app.theme.comment_time)),
                Span::raw("        Toggle bookmark • "),
                Span::styled("B", Style::default().fg(app.theme.comment_time)),
                Span::raw(" View bookmarks"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("H", Style::default().fg(app.theme.comment_time)),
                Span::raw("        View history • "),
                Span::styled("X", Style::default().fg(app.theme.comment_time)),
                Span::raw(" Clear history"),
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
        ],
        _ => {
            // Page 2: Theme Editor
            vec![
                Line::from(vec![Span::styled(
                    "Theme Editor Shortcuts (Tab for General)",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(app.theme.selection_bg),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Toggle Editor",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(app.theme.selection_bg),
                )]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("E", Style::default().fg(app.theme.comment_time)),
                    Span::raw("        Open/close theme editor"),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "When Editor is Active",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(app.theme.selection_bg),
                )]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("↑ / ↓", Style::default().fg(app.theme.comment_time)),
                    Span::raw("      Navigate theme properties"),
                ]),
                Line::from(vec![Span::raw(
                    "            (Background, Foreground, Selection, etc.)",
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("← / →", Style::default().fg(app.theme.comment_time)),
                    Span::raw("      Switch RGB channels (Red/Green/Blue)"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("+ / =", Style::default().fg(app.theme.comment_time)),
                    Span::raw("      Increase color value (+5)"),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("- / _", Style::default().fg(app.theme.comment_time)),
                    Span::raw("      Decrease color value (-5)"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("s", Style::default().fg(app.theme.comment_time)),
                    Span::raw("        Save theme to JSON"),
                ]),
                Line::from(vec![Span::raw(
                    "            (Exports to ./themes/custom_custom.json)",
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled("Esc", Style::default().fg(app.theme.comment_time)),
                    Span::raw("      Close editor (discard changes)"),
                ]),
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "ℹ Note",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(app.theme.comment_time),
                )]),
                Line::from(vec![Span::raw(
                    "  Theme changes apply in real-time as you edit.",
                )]),
                Line::from(vec![Span::raw("  All colors use RGB values (0-255).")]),
            ]
        }
    };

    let p = Paragraph::new(shortcuts)
        .style(Style::default().fg(app.theme.foreground))
        .wrap(Wrap { trim: false }); // Don't trim to preserve indentation

    f.render_widget(p, inner_area);
}

fn render_theme_editor_overlay(app: &App, f: &mut Frame) {
    use crate::internal::ui::theme_editor::{ColorChannel, ThemeProperty};

    let area = f.area();

    // Create centered popup
    let popup_width = 60.min(area.width - 4);
    let popup_height = 20.min(area.height - 4);

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear background
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.selection_bg))
        .title(" Theme Editor ")
        .title_style(
            Style::default()
                .fg(app.theme.selection_fg)
                .bg(app.theme.selection_bg)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(app.theme.background));

    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Split into Property List (Left) and Color Editor (Right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Properties
            Constraint::Percentage(50), // Color Editor
        ])
        .split(inner_area);

    // 1. Property List
    let properties = ThemeProperty::all();
    let items: Vec<ListItem> = properties
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_selected = i == app.theme_editor.selected_property;
            let style = match is_selected {
                true => Style::default()
                    .fg(app.theme.selection_fg)
                    .bg(app.theme.selection_bg)
                    .add_modifier(Modifier::BOLD),
                false => Style::default().fg(app.theme.foreground),
            };
            ListItem::new(format!(" {}", p.name())).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(app.theme.comment_time)),
    );
    f.render_widget(list, chunks[0]);

    // 2. Color Editor
    if let Some(property) = app.theme_editor.get_current_property() {
        let color = property.get_color(&app.theme_editor.temp_theme);

        // Extract RGB values
        let (r, g, b) = match color {
            ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
            _ => (128, 128, 128), // Fallback for non-RGB
        };

        let editor_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Length(3), // Red
                Constraint::Length(3), // Green
                Constraint::Length(3), // Blue
                Constraint::Min(1),    // Preview/Help
            ])
            .margin(1)
            .split(chunks[1]);

        // Header
        f.render_widget(
            Paragraph::new(format!("Editing: {}", property.name())).style(
                Style::default()
                    .fg(app.theme.foreground)
                    .add_modifier(Modifier::BOLD),
            ),
            editor_chunks[0],
        );

        // Helper to render channel slider
        let render_channel =
            |f: &mut Frame, area: Rect, name: &str, value: u8, channel: ColorChannel| {
                let is_selected = app.theme_editor.selected_channel == channel;
                let label_style = match is_selected {
                    true => Style::default()
                        .fg(app.theme.selection_bg)
                        .add_modifier(Modifier::BOLD),
                    false => Style::default().fg(app.theme.foreground),
                };

                let gauge = ratatui::widgets::Gauge::default()
                    .block(Block::default().title(name).title_style(label_style))
                    .gauge_style(
                        Style::default()
                            .fg(match channel {
                                ColorChannel::Red => ratatui::style::Color::Red,
                                ColorChannel::Green => ratatui::style::Color::Green,
                                ColorChannel::Blue => ratatui::style::Color::Blue,
                            })
                            .bg(ratatui::style::Color::DarkGray),
                    )
                    .ratio(value as f64 / 255.0)
                    .label(format!("{}", value));

                f.render_widget(gauge, area);
            };

        render_channel(f, editor_chunks[1], "Red", r, ColorChannel::Red);
        render_channel(f, editor_chunks[2], "Green", g, ColorChannel::Green);
        render_channel(f, editor_chunks[3], "Blue", b, ColorChannel::Blue);

        // Footer: Shortcuts (Left) + Hex/Preview (Right)
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Shortcuts
                Constraint::Percentage(40), // Hex/Preview
            ])
            .split(editor_chunks[4]);

        // Shortcuts
        let help_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "←/→",
                    Style::default()
                        .fg(app.theme.link)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Channel"),
            ]),
            Line::from(vec![
                Span::styled(
                    "+/-",
                    Style::default()
                        .fg(app.theme.link)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Adjust"),
            ]),
            Line::from(vec![
                Span::styled(
                    "s  ",
                    Style::default()
                        .fg(app.theme.link)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Save"),
            ]),
        ];
        f.render_widget(
            Paragraph::new(help_text).style(Style::default().fg(app.theme.foreground)),
            footer_chunks[0],
        );

        // Hex Code and Preview
        let hex_code = format!("#{:02X}{:02X}{:02X}", r, g, b);
        let preview_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Hex Code
                Constraint::Length(1), // Spacing
                Constraint::Length(2), // Preview Box
            ])
            .split(footer_chunks[1]);

        f.render_widget(
            Paragraph::new(hex_code)
                .style(
                    Style::default()
                        .fg(app.theme.comment_time)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Right),
            preview_layout[1],
        );

        f.render_widget(
            Block::default().style(Style::default().bg(color)),
            preview_layout[3],
        );
    }

    // Render Naming Popup if in Naming state
    if let crate::internal::ui::theme_editor::EditorState::Naming = app.theme_editor.state {
        let area = {
            let area = f.area();
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(10),
                    Constraint::Percentage(40),
                ])
                .split(area);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(vertical[1])[1]
        };
        f.render_widget(Clear, area);

        let block = Block::default()
            .title(" Save Theme ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.selection_bg));

        f.render_widget(block.clone(), area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .margin(1)
            .split(area);

        f.render_widget(
            Paragraph::new("Enter theme name:").style(Style::default().fg(app.theme.foreground)),
            layout[0],
        );

        f.render_widget(
            Paragraph::new(app.theme_editor.name_input.as_str()).style(
                Style::default()
                    .fg(app.theme.foreground)
                    .add_modifier(Modifier::BOLD),
            ),
            layout[1],
        );

        // Cursor
        f.set_cursor_position((
            layout[1].x + app.theme_editor.name_input.len() as u16,
            layout[1].y,
        ));

        f.render_widget(
            Paragraph::new("Enter: Save | Esc: Cancel")
                .style(Style::default().fg(app.theme.comment_time)),
            layout[2],
        );
    }
}

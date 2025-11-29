use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub message: String,
}

pub struct LogViewer {
    pub visible: bool,
    pub entries: Vec<LogEntry>,
    pub scroll: u16,
    pub active_tab: usize,
    pub tabs: Vec<String>,
    #[allow(dead_code)]
    pub filter_level: Option<String>,
    #[allow(dead_code)]
    pub filter_module: Option<String>,
    pub log_path: String,
}

impl LogViewer {
    pub fn new(log_dir: String) -> Self {
        Self {
            visible: false,
            entries: Vec::new(),
            scroll: 0,
            active_tab: 0,
            tabs: vec!["Logs".to_string(), "Metrics".to_string()],
            filter_level: None,
            filter_module: None,
            log_path: log_dir,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.load_logs();
            // Auto-scroll to bottom when opening
            self.scroll_to_bottom();
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
        // Reset scroll when switching tabs
        self.scroll_to_bottom();
    }

    pub fn load_logs(&mut self) {
        // Construct filename with current UTC date to match tracing-appender's daily rotation
        // Format: tui-hn-app.log.YYYY-MM-DD
        let date_str = jiff::Zoned::now()
            .with_time_zone(jiff::tz::TimeZone::UTC)
            .strftime("%Y-%m-%d")
            .to_string();
        let filename = format!("tui-hn-app.log.{}", date_str);
        let path = Path::new(&self.log_path).join(filename);

        if !path.exists() {
            return;
        }

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            // Simple regex for parsing standard tracing output
            // Example: 2025-11-29T09:30:15.123Z INFO app: App initialized
            let re = Regex::new(
                r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+(\w+)\s+([^:]+):\s+(.*)$",
            )
            .unwrap();

            // Read last 1000 lines efficiently (simplified for now: read all, keep last 1000)
            // For a real production app, we'd seek from end, but this is fine for now.
            let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

            // Regex to strip ANSI escape codes
            let ansi_re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();

            self.entries = lines
                .iter()
                .rev()
                .take(1000)
                .rev()
                .map(|line| {
                    // Strip ANSI codes first
                    let clean_line = ansi_re.replace_all(line, "").to_string();

                    if let Some(caps) = re.captures(&clean_line) {
                        LogEntry {
                            timestamp: caps[1].to_string(),
                            level: caps[2].to_string(),
                            module: caps[3].to_string(),
                            message: caps[4].to_string(),
                        }
                    } else {
                        // Fallback for lines that don't match (e.g. panic traces)
                        LogEntry {
                            timestamp: "".to_string(),
                            level: "UNKNOWN".to_string(),
                            module: "".to_string(),
                            message: clean_line,
                        }
                    }
                })
                .collect();
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        // Simple bound check, can be improved
        self.scroll += 1;
    }

    pub fn scroll_to_bottom(&mut self) {
        let count = self.filtered_entries().len();
        if count > 20 {
            self.scroll = (count - 20) as u16;
        } else {
            self.scroll = 0;
        }
    }

    fn filtered_entries(&self) -> Vec<&LogEntry> {
        match self.active_tab {
            0 => self.entries.iter().collect(), // Logs: show all
            1 => self
                .entries
                .iter()
                .filter(|e| e.message.contains("duration_ms") || e.message.contains("elapsed"))
                .collect(), // Metrics: show only performance logs
            _ => Vec::new(),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Clear background for overlay
        use ratatui::widgets::Clear;

        // Center the overlay (90% width, 80% height)
        let width = area.width * 90 / 100;
        let height = area.height * 80 / 100;
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay_area = Rect::new(x, y, width, height);
        f.render_widget(Clear, overlay_area);

        // 1. Render Outer Block (Borders::ALL)
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .title("Log Viewer (Tab: Switch, Esc: Close)");
        f.render_widget(outer_block.clone(), overlay_area);

        // 2. Calculate Inner Layout
        let inner_area = outer_block.inner(overlay_area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tabs
                Constraint::Length(1), // Separator Line
                Constraint::Min(0),    // Logs Content
            ])
            .split(inner_area);

        let tabs_area = chunks[0];
        let separator_area = chunks[1];
        let logs_area = chunks[2];

        // 3. Render Tabs
        use ratatui::widgets::Tabs;
        let titles: Vec<Line> = self
            .tabs
            .iter()
            .map(|t| Line::from(Span::styled(t, Style::default().fg(Color::Green))))
            .collect();

        let tabs = Tabs::new(titles)
            .select(self.active_tab)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(" | ")
            // Add horizontal padding to tabs so they don't touch the left border
            .block(Block::default().padding(Padding::horizontal(1)));
        f.render_widget(tabs, tabs_area);

        // 4. Render Separator Line
        // We construct a Rect that spans the FULL width of the overlay (including borders)
        // but is positioned at the vertical level of the separator area.
        let full_width_separator_area =
            Rect::new(overlay_area.x, separator_area.y, overlay_area.width, 1);

        // Custom border set to use T-junctions (├ and ┤) instead of corners
        // We inherit from PLAIN to get the horizontal line symbol.
        let border_set = symbols::border::Set {
            top_left: symbols::line::VERTICAL_RIGHT,
            top_right: symbols::line::VERTICAL_LEFT,
            ..symbols::border::PLAIN
        };

        let separator_block = Block::default()
            .borders(Borders::TOP)
            .border_set(border_set)
            .border_style(Style::default().fg(Color::DarkGray));

        f.render_widget(separator_block, full_width_separator_area);

        // 5. Render Logs Content
        let filtered_entries = self.filtered_entries();
        let log_lines: Vec<Line> = filtered_entries
            .iter()
            .skip(self.scroll as usize)
            .map(|entry| {
                let level_style = match entry.level.as_str() {
                    "ERROR" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    "WARN" => Style::default().fg(Color::Yellow),
                    "INFO" => Style::default().fg(Color::Blue),
                    "DEBUG" => Style::default().fg(Color::Green),
                    "TRACE" => Style::default().fg(Color::Magenta),
                    _ => Style::default(),
                };

                Line::from(vec![
                    Span::styled(
                        format!("{} ", entry.timestamp),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(format!("{:5} ", entry.level), level_style),
                    Span::styled(
                        format!("{}: ", entry.module),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(&entry.message),
                ])
            })
            .collect();

        let logs = Paragraph::new(log_lines)
            .wrap(Wrap { trim: false })
            // Add horizontal padding to logs so they don't touch the borders
            .block(Block::default().padding(Padding::horizontal(1)));

        f.render_widget(&logs, logs_area);
    }
}

use ratatui::{Terminal, backend::TestBackend};
use tui_hn_app::internal::ui::log_viewer::LogViewer;

#[test]
fn test_log_viewer_render() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut log_viewer = LogViewer::new("logs".to_string());
    log_viewer.visible = true;
    // Add some dummy logs if possible, or just test empty state
    // Since we can't easily inject logs without writing to file, we'll test the empty structure

    terminal
        .draw(|f| {
            let area = f.area();
            log_viewer.render(f, area);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    insta::assert_debug_snapshot!(buffer);
}

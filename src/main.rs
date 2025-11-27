mod api;
mod app;
mod config;
mod internal;
mod tui;
mod utils;

// Legacy state module - to be removed or refactored
// mod state;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Try to initialize the terminal first so we can decide where tracing should write.
    // When the TUI is running we must avoid writing logs to stderr/stdout (which would
    // corrupt the UI). In that case we write logs to a rotating file. If TUI init fails
    // we enable console logging so messages are visible to the user.
    match tui::init() {
        Ok(terminal) => {
            // Running TUI: log to a daily rotating file (logs/tui-hn-app.log).
            // Use the non-blocking appender to avoid blocking the UI thread.
            let file_appender = tracing_appender::rolling::daily("logs", "tui-hn-app.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .with_writer(non_blocking)
                .init();

            // Start the application using the terminal we successfully initialized.
            let mut app = App::new();
            let res = app.run(terminal).await;

            // Restore terminal state before exiting so the console is usable again.
            tui::restore()?;

            if let Err(err) = res {
                // Print a short error to stderr as well so it's visible if someone runs the binary
                // directly; detailed traces will be available in the log file.
                eprintln!("{err:?}");
            }

            Ok(())
        }
        Err(e) => {
            // Failed to initialize TUI: enable console logging so messages are visible.
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .init();

            eprintln!("Failed to initialize TUI: {e:?}");
            // Convert the underlying error into an anyhow::Error for the main return type.
            Err(e.into())
        }
    }
}

mod api;
mod config;
mod internal;
mod tui;
mod utils;

use anyhow::Result;
use internal::ui::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration first to get logging settings
    let config = config::AppConfig::load();

    // Try to initialize the terminal first so we can decide where tracing should write.
    // When the TUI is running we must avoid writing logs to stderr/stdout (which would
    // corrupt the UI). In that case we write logs to a rotating file. If TUI init fails
    // we enable console logging so messages are visible to the user.
    match tui::init() {
        Ok(terminal) => {
            // Running TUI: log to a daily rotating file.
            // Use configured directory or default to "logs"
            let log_dir = config.logging.log_directory.as_deref().unwrap_or("logs");
            let file_appender = tracing_appender::rolling::daily(log_dir, "tui-hn-app.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            // Build EnvFilter
            // If RUST_LOG is set, it takes precedence.
            // Otherwise, build from config.
            let env_filter = match std::env::var("RUST_LOG") {
                Ok(_) => tracing_subscriber::EnvFilter::from_default_env(),
                Err(_) => {
                    let mut filter_str = config.logging.level.to_string();
                    for (module, level) in &config.logging.module_levels {
                        filter_str.push_str(&format!(",{}={}", module, level));
                    }
                    tracing_subscriber::EnvFilter::new(filter_str)
                }
            };

            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_writer(non_blocking)
                .with_ansi(false)
                .compact()
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

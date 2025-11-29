use std::time::{Duration, Instant};

/// Type of notification to display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    #[allow(dead_code)]
    Warning,
    Error,
}

impl NotificationType {
    fn timeout(&self) -> Duration {
        match self {
            NotificationType::Info => Duration::from_secs(3),
            NotificationType::Warning => Duration::from_secs(5),
            NotificationType::Error => Duration::from_secs(10),
        }
    }
}

/// A notification message with type and auto-dismiss capability
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: Instant,
}

impl Notification {
    /// Create a new info notification with default 3s auto-dismiss
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Info)
    }

    /// Create a new warning notification with default 5s auto-dismiss
    #[allow(dead_code)]
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Warning)
    }

    /// Create a new error notification with default 10s auto-dismiss
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Error)
    }

    fn new(message: impl Into<String>, notification_type: NotificationType) -> Self {
        Self {
            message: message.into(),
            notification_type,
            timestamp: Instant::now(),
        }
    }

    /// Check if this notification should be auto-dismissed
    pub fn should_dismiss(&self) -> bool {
        self.timestamp.elapsed() > self.notification_type.timeout()
    }

    #[allow(dead_code)]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[allow(dead_code)]
    pub fn notification_type(&self) -> NotificationType {
        self.notification_type
    }

    /// Get the remaining time before auto-dismiss
    #[allow(dead_code)]
    pub fn remaining_time(&self) -> Duration {
        self.notification_type
            .timeout()
            .saturating_sub(self.timestamp.elapsed())
    }
}

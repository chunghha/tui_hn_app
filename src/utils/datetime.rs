use std::time::{SystemTime, UNIX_EPOCH};

/// Format a unix timestamp (seconds) into a short relative string like "2d ago",
/// "3h ago", "15m ago", or "just now".
pub fn format_timestamp(timestamp: &i64) -> String {
    // Get current time as seconds since unix epoch
    let now_secs = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(dur) => dur.as_secs() as i64,
        Err(_) => return "unknown".to_string(),
    };

    let ts = *timestamp;

    // If timestamp is in the future or invalid, show just now
    if now_secs <= ts {
        return "just now".to_string();
    }

    let delta = now_secs - ts;
    let days = delta / 86_400;
    if days > 0 {
        return format!("{}d ago", days);
    }

    let hours = delta / 3_600;
    if hours > 0 {
        return format!("{}h ago", hours);
    }

    let minutes = delta / 60;
    if minutes > 0 {
        return format!("{}m ago", minutes);
    }

    "just now".to_string()
}

#[cfg(test)]
mod tests {
    use super::format_timestamp;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Helper to get current unix seconds
    fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64
    }

    #[test]
    fn returns_just_now_for_current_and_future() {
        let now = now_secs();
        // current timestamp -> just now
        assert_eq!(format_timestamp(&now), "just now");
        // future timestamp -> just now (per implementation)
        assert_eq!(format_timestamp(&(now + 10)), "just now");
    }

    #[test]
    fn returns_minutes_hours_and_days() {
        let now = now_secs();

        // Less than a minute ago -> just now
        assert_eq!(format_timestamp(&(now - 30)), "just now");

        // 5 minutes ago
        assert_eq!(format_timestamp(&(now - 5 * 60)), "5m ago");

        // 2 hours ago
        assert_eq!(format_timestamp(&(now - 2 * 3_600)), "2h ago");

        // 3 days ago
        assert_eq!(format_timestamp(&(now - 3 * 86_400)), "3d ago");
    }
}

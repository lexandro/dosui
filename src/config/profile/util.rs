//! Small time helpers used by the library view.

/// Current Unix time in seconds (0 if the clock is before the epoch).
pub fn now_unix() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A coarse human-readable "time ago" string for a past Unix timestamp.
pub fn humanize_since(now: u64, then: u64) -> String {
    if then > now {
        return "just now".to_string();
    }
    let secs = now - then;
    match secs {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", secs / 60),
        3600..=86_399 => format!("{} h ago", secs / 3600),
        86_400..=172_799 => "yesterday".to_string(),
        _ => format!("{} days ago", secs / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize_since_buckets() {
        assert_eq!(humanize_since(100, 100), "just now");
        assert_eq!(humanize_since(1000, 700), "5 min ago");
        assert_eq!(humanize_since(10_000, 3_000), "1 h ago");
        assert_eq!(humanize_since(200_000, 100_000), "yesterday"); // ~27h
        assert_eq!(humanize_since(300_000, 100_000), "2 days ago");
        assert_eq!(humanize_since(50, 100), "just now"); // clock skew -> safe
    }
}

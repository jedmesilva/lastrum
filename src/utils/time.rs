//! Time utility functions
//! 
//! This module provides utility functions for working with
//! timestamps and durations.

use chrono::{DateTime, Duration, Utc};

/// Get the current timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Format a timestamp as ISO 8601
pub fn format_iso8601(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Parse an ISO 8601 timestamp
pub fn parse_iso8601(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
}

/// Check if a timestamp is older than a specified duration
pub fn is_older_than(dt: &DateTime<Utc>, duration: Duration) -> bool {
    let now = Utc::now();
    now.signed_duration_since(*dt) > duration
}

/// Calculate the age of a timestamp
pub fn age(dt: &DateTime<Utc>) -> Duration {
    Utc::now().signed_duration_since(*dt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration as StdDuration;
    
    #[test]
    fn test_now() {
        let t1 = now();
        sleep(StdDuration::from_millis(10));
        let t2 = now();
        assert!(t2 > t1);
    }
    
    #[test]
    fn test_format_parse_iso8601() {
        let now = now();
        let formatted = format_iso8601(&now);
        let parsed = parse_iso8601(&formatted).unwrap();
        assert_eq!(now, parsed);
    }
    
    #[test]
    fn test_is_older_than() {
        let old = now() - Duration::seconds(60);
        assert!(is_older_than(&old, Duration::seconds(30)));
        assert!(!is_older_than(&old, Duration::seconds(120)));
    }
}

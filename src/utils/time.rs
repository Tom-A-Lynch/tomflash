use chrono::{DateTime, Duration, TimeZone, Utc};
use rand::Rng;
use std::ops::Range;

use super::{Result, UtilError};

/// Generates random intervals for posting simulation
pub fn generate_posting_interval(base_interval: Duration) -> Duration {
    let mut rng = rand::thread_rng();
    let variance = base_interval.num_seconds() as f64 * 0.2; // 20% variance
    let variance_seconds = rng.gen_range(-variance..variance);
    
    Duration::seconds((base_interval.num_seconds() as f64 + variance_seconds) as i64)
}

/// Checks if current time is within active hours (configured for agent's timezone)
pub fn is_active_hours() -> bool {
    let now = Utc::now();
    let hour = now.hour();
    
    // Active between 8 AM and 2 AM
    (8..=23).contains(&hour) || (0..=2).contains(&hour)
}

/// Formats a datetime for consistent display
pub fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Parses a datetime string with flexible format
pub fn parse_datetime(datetime_str: &str) -> Result<DateTime<Utc>> {
    // Try common formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.3fZ",
        "%Y-%m-%d",
    ];

    for format in formats {
        if let Ok(dt) = DateTime::parse_from_str(datetime_str, format) {
            return Ok(dt.with_timezone(&Utc));
        }
    }

    Err(UtilError::TimeError(format!("Unable to parse datetime: {}", datetime_str)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posting_interval() {
        let base = Duration::minutes(30);
        let interval = generate_posting_interval(base);
        
        // Should be within 20% of base
        assert!(interval >= base * 0.8);
        assert!(interval <= base * 1.2);
    }

    #[test]
    fn test_datetime_parsing() {
        let test_cases = [
            "2024-03-14 12:00:00",
            "2024-03-14T12:00:00.000Z",
            "2024-03-14",
        ];

        for case in test_cases {
            assert!(parse_datetime(case).is_ok());
        }
    }
}
//! # Utility Functions
//!
//! This module provides helper functions for parsing and formatting durations
//! and other common tasks used throughout albumseq_cli.
//!
//! ## Example
//! ```rust
//! let dur = parse_duration("3:45").unwrap();
//! let s = format_duration(dur);
//! ```

use albumseq::Duration;

/// Formats a duration in minutes (f64) as "MM:SS".
///
/// # Arguments
/// * `duration` - The duration in minutes.
///
/// # Returns
/// A string in "MM:SS" format.
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = (duration * 60.0).round() as u64;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

/// Parses a duration from "MM:SS" or decimal minutes.
///
/// # Arguments
/// * `s` - The input string.
///
/// # Returns
/// `Some(f64)` if parsing succeeds, or `None` if the input is invalid.
pub fn parse_duration(s: &str) -> Option<f64> {
    if let Some((min_str, sec_str)) = s.split_once(':') {
        if let (Ok(min), Ok(sec)) = (min_str.parse::<u32>(), sec_str.parse::<u32>()) {
            return Some(min as f64 + sec as f64 / 60.0);
        }
    }
    s.parse::<f64>().ok()
}

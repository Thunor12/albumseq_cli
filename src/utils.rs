use albumseq::Duration;

/// Format a duration in minutes (f64) as "MM:SS"
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = (duration * 60.0).round() as u64;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

/// Parse a duration from "MM:SS" or decimal minutes
pub fn parse_duration(s: &str) -> Option<f64> {
    if let Some((min_str, sec_str)) = s.split_once(':') {
        if let (Ok(min), Ok(sec)) = (min_str.parse::<u32>(), sec_str.parse::<u32>()) {
            return Some(min as f64 + sec as f64 / 60.0);
        }
    }
    s.parse::<f64>().ok()
}
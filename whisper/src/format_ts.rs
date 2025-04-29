use chrono::DateTime;

pub fn format_ts(ts: i64, time_format: &str) -> String {
    match DateTime::from_timestamp(ts, 0) {
        Some(dt) => dt.format(time_format).to_string(),
        None => ts.to_string(),
    }
}

pub fn display_ts(ts: i64, time_format: Option<&str>) -> String {
    match time_format {
        Some(time_format) => format_ts(ts, time_format),
        _ => ts.to_string(),
    }
}

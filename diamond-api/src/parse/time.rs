use crate::error::ParseError;
use serde::{Deserialize, Deserializer};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn de_time_parse<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    time_parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
}

pub fn time_parse(s: String) -> Result<u32, ParseError> {
    if s.starts_with('-') {
        // Relative time
        let (multi, count) = match &s.chars().last().unwrap() {
            's' => (1, 1),
            'h' => (3600, 1),
            'd' => (3600 * 24, 1),
            'w' => (3600 * 24 * 7, 1),
            'y' => (3600 * 24 * 365, 1),
            'n' if s.ends_with("min") => (60, 3),
            'n' if s.ends_with("mon") => (3600 * 24 * 30, 3),
            _ => return Err(ParseError::Time),
        };

        let s2 = &s[1..s.len() - count];
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;

        let v = now - s2.parse::<u32>()? * multi;
        Ok(v)
    } else {
        // Absolute time
        match s.as_str() {
            "now" => Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32),
            "yesterday" => {
                Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32 - 3600 * 24)
            }
            "" => Err(ParseError::EmptyString),
            // Unix timestamp parse as default
            _ => {
                // Unix timestamp
                Ok(s.parse::<u32>()?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_de_time_parse_ok() {
        let mut de = serde_json::Deserializer::new(serde_json::de::StrRead::new("\"123\""));
        let ts = de_time_parse(&mut de).unwrap();
        assert_eq!(ts, 123_u32);
    }

    #[test]
    fn test_de_time_parse_error() {
        let mut de = serde_json::Deserializer::new(serde_json::de::StrRead::new("\"lol\""));
        let err = de_time_parse(&mut de).unwrap_err();
        assert_eq!(err.to_string().as_str(), "invalid digit found in string");
    }
}

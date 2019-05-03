use std::str::FromStr;
use regex::Regex;
use lazy_static::lazy_static;
use serde::*;

fn get_unit_multiplier(s: &str) -> Result<u32, String> {
    if s.is_empty() || "seconds".starts_with(s) {
        Ok(1)
    } else if "minutes".starts_with(s) {
        Ok(60)
    } else if "hours".starts_with(s) {
        Ok(3600)
    } else if "days".starts_with(s) {
        Ok(86400)
    } else if "weeks".starts_with(s) {
        Ok(86400 * 7)
    } else if "years".starts_with(s) {
        Ok(86400 * 365)
    } else {
        Err(format!("Invalid unit '{}'", s))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Retention {
    pub seconds_per_point: u32,
    pub points: u32,
}

impl Retention {
    pub fn retention(self) -> u32 {
        self.seconds_per_point * self.points
    }
}

impl FromStr for Retention {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RETENTION_DEF_RE: Regex = Regex::new(r#"(?i)^\s*((\d+)([a-z]*)):((\d+)([a-z]*))\s*$"#).unwrap();
        }

        let captures = RETENTION_DEF_RE.captures(s).ok_or_else(|| format!("Invalid retention definition '{}'", s))?;

        let mut precision = u32::from_str_radix(captures.get(2).unwrap().as_str(), 10).unwrap();
        if !captures.get(3).unwrap().as_str().is_empty() {
            precision *= get_unit_multiplier(captures.get(3).unwrap().as_str())?;
        }

        let mut points = u32::from_str_radix(captures.get(5).unwrap().as_str(), 10).unwrap();
        if !captures.get(6).unwrap().as_str().is_empty() {
            points = points * get_unit_multiplier(captures.get(6).unwrap().as_str())? / precision;
        }

        Ok(Self { seconds_per_point: precision, points })
    }
}

impl<'de> Deserialize<'de> for Retention {
    fn deserialize<D>(deserializer: D) -> Result<Retention, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seq = <[u32; 2]>::deserialize(deserializer)?;
        Ok(Retention {
            seconds_per_point: seq[0],
            points: seq[1],
        })
    }
}

pub fn parse_duration(s: &str) -> Result<u32, String> {
    lazy_static! {
        static ref RETENTION_DEF_RE: Regex = Regex::new(r#"(?i)^\s*(\d+)([a-z]*)\s*$"#).unwrap();
    }

    let captures = RETENTION_DEF_RE.captures(s).ok_or_else(|| format!("Invalid duration definition '{}'", s))?;

    let mut precision = u32::from_str_radix(captures.get(1).unwrap().as_str(), 10).unwrap();
    if !captures.get(2).unwrap().as_str().is_empty() {
        precision *= get_unit_multiplier(captures.get(2).unwrap().as_str())?;
    }

    if precision == 0 {
        Err("Precision cannot be zero".to_owned())
    } else {
        Ok(precision)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_multiplier() {
        assert_eq!(get_unit_multiplier(""), Ok(1));
        assert_eq!(get_unit_multiplier("s"), Ok(1));
        assert_eq!(get_unit_multiplier("m"), Ok(60));
        assert_eq!(get_unit_multiplier("h"), Ok(3600));
        assert_eq!(get_unit_multiplier("d"), Ok(86400));
        assert_eq!(get_unit_multiplier("w"), Ok(604800));
        assert_eq!(get_unit_multiplier("y"), Ok(31536000));
        assert_eq!(get_unit_multiplier("z"), Err("Invalid unit 'z'".to_string()));
    }

    #[test]
    fn test_valid_retentions() {
        assert_eq!("60:10".parse(), Ok(Retention { seconds_per_point: 60, points: 10 }));
        assert_eq!("10:60".parse(), Ok(Retention { seconds_per_point: 10, points: 60 }));
        assert_eq!("10s:10h".parse(), Ok(Retention { seconds_per_point: 10, points: 3600 }));
    }

    #[test]
    fn test_invalid_retentions() {
        assert_eq!("10".parse::<Retention>(), Err("Invalid retention definition '10'".to_string()));
        assert_eq!("10:10$".parse::<Retention>(), Err("Invalid retention definition '10:10$'".to_string()));

        assert_eq!("10x:10".parse::<Retention>(), Err("Invalid unit 'x'".to_string()));
        assert_eq!("60:10x".parse::<Retention>(), Err("Invalid unit 'x'".to_string()));
        assert_eq!("10X:10".parse::<Retention>(), Err("Invalid unit 'X'".to_string()));
    }


    #[test]
    fn test_valid_precision() {
        assert_eq!(parse_duration("10"), Ok(10));
        assert_eq!(parse_duration("60"), Ok(60));
        assert_eq!(parse_duration("10h"), Ok(36000));
    }

    #[test]
    fn test_invalid_precision() {
        assert!(parse_duration("10$").is_err());
        assert!(parse_duration("-10").is_err());
        assert!(parse_duration("0").is_err());
    }
}

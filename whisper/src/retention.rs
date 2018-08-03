use std::str::FromStr;
use regex::Regex;

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

pub fn parse_duration(s: &str) -> Result<u32, String> {
    lazy_static! {
        static ref RETENTION_DEF_RE: Regex = Regex::new(r#"(?i)^\s*(\d+)([a-z]*)\s*$"#).unwrap();
    }

    let captures = RETENTION_DEF_RE.captures(s).ok_or_else(|| format!("Invalid duration definition '{}'", s))?;

    let mut precision = u32::from_str_radix(captures.get(1).unwrap().as_str(), 10).unwrap();
    if !captures.get(2).unwrap().as_str().is_empty() {
        precision *= get_unit_multiplier(captures.get(2).unwrap().as_str())?;
    }

    Ok(precision)
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
}

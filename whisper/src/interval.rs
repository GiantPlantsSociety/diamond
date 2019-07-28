#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    from: u32,
    until: u32,
}

impl Interval {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(from: u32, until: u32) -> Result<Self, String> {
        if from <= until {
            Ok(Self { from, until })
        } else {
            Err(format!(
                "Invalid time interval: from time '{}' is after until time '{}'.",
                from, until
            ))
        }
    }

    pub fn past(until: u32, duration: u32) -> Self {
        Self {
            from: until - duration,
            until,
        }
    }

    pub fn from(self) -> u32 {
        self.from
    }

    pub fn until(self) -> u32 {
        self.until
    }

    pub fn contains(self, other: Interval) -> bool {
        self.from <= other.from && other.until <= self.until
    }

    pub fn intersects(self, other: Interval) -> bool {
        self.from <= other.until && self.until >= other.from
    }

    pub fn intersection(self, other: Interval) -> Result<Interval, String> {
        Interval::new(
            u32::max(self.from, other.from),
            u32::min(self.until, other.until),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_valid() {
        assert_eq!(Interval::new(1, 2), Ok(Interval { from: 1, until: 2 }));
        assert_eq!(Interval::new(2, 2), Ok(Interval { from: 2, until: 2 }));
    }

    #[test]
    fn interval_invalid() {
        assert!(Interval::new(2, 1).is_err());
    }

    #[test]
    fn is_contains() -> Result<(), String> {
        let check = Interval::new(3, 10)?.contains(Interval::new(4, 9)?);
        assert!(check);
        Ok(())
    }

    #[test]
    fn is_not_contains() -> Result<(), String> {
        let check1 = Interval::new(5, 10)?.contains(Interval::new(4, 9)?);
        assert!(!check1);

        let check2 = Interval::new(5, 10)?.contains(Interval::new(6, 11)?);
        assert!(!check2);

        let check3 = Interval::new(5, 10)?.contains(Interval::new(4, 11)?);
        assert!(!check3);

        Ok(())
    }
}

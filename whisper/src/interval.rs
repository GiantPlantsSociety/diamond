#[derive(Debug, Clone, Copy)]
pub struct Interval {
    from: u32,
    until: u32,
}

impl Interval {
    pub fn new(from: u32, until: u32) -> Result<Self, String> {
        if from <= until {
            Ok(Self { from, until })
        } else {
            Err(format!("Invalid time interval: from time '{}' is after until time '{}'.", from, until))
        }
    }

    pub fn past(until: u32, duration: u32) -> Self {
        Self { from: until - duration, until }
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
            u32::min(self.until, other.until)
        )
    }
}

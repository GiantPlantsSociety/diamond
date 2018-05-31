use std::cmp;
use std::str::FromStr;

fn cmp_f64(a: &f64, b: &f64) -> cmp::Ordering {
    a.partial_cmp(b).unwrap_or(::std::cmp::Ordering::Equal)
}

fn cmp_f64_abs(a: &f64, b: &f64) -> cmp::Ordering {
    cmp_f64(&a.abs(), &b.abs())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AggregationMethod {
    Average,
    Sum,
    Last,
    Max,
    Min,
    AvgZero,
    AbsMax,
    AbsMin,
}

impl AggregationMethod {
    pub fn from_type(aggregation_type: u32) -> Option<Self> {
        match aggregation_type {
            1 => Some(AggregationMethod::Average),
            2 => Some(AggregationMethod::Sum),
            3 => Some(AggregationMethod::Last),
            4 => Some(AggregationMethod::Max),
            5 => Some(AggregationMethod::Min),
            6 => Some(AggregationMethod::AvgZero),
            7 => Some(AggregationMethod::AbsMax),
            8 => Some(AggregationMethod::AbsMin),
            _ => None,
        }
    }

    pub fn to_type(&self) -> u32 {
        match self {
            AggregationMethod::Average => 1,
            AggregationMethod::Sum => 2,
            AggregationMethod::Last => 3,
            AggregationMethod::Max => 4,
            AggregationMethod::Min => 5,
            AggregationMethod::AvgZero => 6,
            AggregationMethod::AbsMax => 7,
            AggregationMethod::AbsMin => 8,
        }
    }

    pub fn aggregate(&self, values: &[Option<f64>]) -> Result<f64, &'static str> {
        match self {
            AggregationMethod::Average => {
                let sum: f64 = values.iter().filter_map(|v| *v).sum();
                let count = values.iter().filter_map(|v| *v).count();
                Ok(sum / count as f64)
            },
            AggregationMethod::Sum => {
                let sum: f64 = values.iter().filter_map(|v| *v).sum();
                Ok(sum)
            },
            AggregationMethod::Last => {
                if let Some(Some(v)) = values.iter().rev().find(|v| v.is_some()) {
                    Ok(*v)
                } else {
                    Err("Empty list of values")
                }
            },
            AggregationMethod::Max => {
                values.iter().filter_map(|v| *v)
                    .max_by(cmp_f64)
                    .ok_or("Empty list of values")
            },
            AggregationMethod::Min => {
                values.iter().filter_map(|v| *v)
                    .min_by(cmp_f64)
                    .ok_or("Empty list of values")
            },
            AggregationMethod::AvgZero => {
                let sum: f64 = values.iter().filter_map(|v| *v).sum();
                let len = values.len();
                Ok(sum / len as f64)
            },
            AggregationMethod::AbsMax => {
                values.iter().filter_map(|v| *v)
                    .max_by(cmp_f64_abs)
                    .ok_or("Empty list of values")
            },
            AggregationMethod::AbsMin => {
                values.iter().filter_map(|v| *v)
                    .min_by(cmp_f64_abs)
                    .ok_or("Empty list of values")
            },
        }
    }
}

impl ::std::default::Default for AggregationMethod {
    fn default() -> Self {
        AggregationMethod::Average
    }
}

impl FromStr for AggregationMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "average" => Ok(AggregationMethod::Average),
            "sum" => Ok(AggregationMethod::Sum),
            "last" => Ok(AggregationMethod::Last),
            "max" => Ok(AggregationMethod::Max),
            "min" => Ok(AggregationMethod::Min),
            "avg_zero" => Ok(AggregationMethod::AvgZero),
            "absmax" => Ok(AggregationMethod::AbsMax),
            "absmin" => Ok(AggregationMethod::AbsMin),
            _ => Err(format!("Unsupported aggregation method '{}'.", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate() {
        assert_eq!(AggregationMethod::Average.aggregate(&[Some(1.0), None, Some(2.0), None, Some(3.0), None, None, Some(4.0)]), Ok(2.5));
        assert_eq!(AggregationMethod::Min.aggregate(&[Some(1.0), None, Some(2.0), None, Some(3.0), None, None, Some(4.0)]), Ok(1.0));
        assert_eq!(AggregationMethod::Max.aggregate(&[Some(1.0), None, Some(2.0), None, Some(3.0), None, None, Some(4.0)]), Ok(4.0));
        assert_eq!(AggregationMethod::Last.aggregate(&[Some(1.0), None, Some(2.0), None, Some(3.0), None, None, Some(4.0)]), Ok(4.0));
        assert_eq!(AggregationMethod::Last.aggregate(&[Some(1.0), None, Some(2.0), None, Some(3.0), None, Some(4.0), None]), Ok(4.0));
        assert_eq!(AggregationMethod::Sum.aggregate(&[Some(10.0), None, Some(2.0), None, Some(3.0), None, None, Some(4.0)]), Ok(19.0));
        assert_eq!(AggregationMethod::AvgZero.aggregate(&[Some(1.0), Some(2.0), Some(3.0), Some(4.0), None, None, None, None]), Ok(1.25));
        assert_eq!(AggregationMethod::AbsMax.aggregate(&[Some(-3.0), Some(-2.0), Some(1.0), Some(2.0)]), Ok(-3.0));
        assert_eq!(AggregationMethod::AbsMax.aggregate(&[Some(-2.0), Some(-1.0), Some(2.0), Some(3.0)]), Ok(3.0));
        assert_eq!(AggregationMethod::AbsMin.aggregate(&[Some(-3.0), Some(-2.0), Some(1.0), Some(2.0)]), Ok(1.0));
        assert_eq!(AggregationMethod::AbsMin.aggregate(&[Some(-2.0), Some(-1.0), Some(2.0), Some(3.0)]), Ok(-1.0));
    }
}

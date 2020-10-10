use serde::*;
use std::str::FromStr;
use whisper::interval::Interval;

pub use crate::error::ResponseError;
pub use crate::render_target::ast::{PathExpression, PathWord};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct MetricName(pub Vec<String>);

impl MetricName {
    pub fn join(&self, item: impl Into<String>) -> Self {
        let mut result = self.clone();
        result.0.push(item.into());
        result
    }
}

impl FromStr for MetricName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.split('.').map(ToString::to_string).collect()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderPoint(pub Option<f64>, pub u32);

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MetricResponseLeaf {
    pub name: MetricName,
    pub is_leaf: bool,
}

pub struct StorageResponse {
    pub name: MetricName,
    pub data: Vec<RenderPoint>,
}

pub trait Storage {
    fn find(
        &self,
        path_expression: &PathExpression,
    ) -> Result<Vec<MetricResponseLeaf>, ResponseError>;

    fn query(
        &self,
        path_expression: &PathExpression,
        interval: Interval,
        now: u64,
    ) -> Result<Vec<StorageResponse>, ResponseError>;
}

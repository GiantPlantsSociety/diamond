use whisper::interval::Interval;

use crate::error::ResponseError;
use crate::render_target::PathExpression;
use crate::storage::{MetricResponseLeaf, RenderPoint, Storage, StorageResponse};

#[derive(Clone)]
pub struct ConstStorage(pub Vec<RenderPoint>);

impl Storage for ConstStorage {
    fn find(
        &self,
        _path_expression: &PathExpression,
    ) -> Result<Vec<MetricResponseLeaf>, ResponseError> {
        Ok(vec![MetricResponseLeaf {
            name: "i.am.a.metric".parse().unwrap(),
            is_leaf: true,
        }])
    }

    fn query(
        &self,
        _path_expression: &PathExpression,
        _interval: Interval,
        _now: u64,
    ) -> Result<Vec<StorageResponse>, ResponseError> {
        Ok(vec![StorageResponse {
            name: "i.am.a.metric".parse().unwrap(),
            data: self.0.clone(),
        }])
    }
}

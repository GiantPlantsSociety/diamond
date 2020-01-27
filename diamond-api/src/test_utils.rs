use actix_web::error::Error;
use glob::Pattern;
use std::path::Path;
use whisper::interval::Interval;

use crate::error::ResponseError;
use crate::storage::{MetricResponseLeaf, RenderPoint, Walker};

#[derive(Clone)]
pub struct WalkerConst(pub Vec<RenderPoint>);

impl Walker for WalkerConst {
    fn walk(&self, _: &str, _: Interval) -> Result<Vec<RenderPoint>, ResponseError> {
        Ok(self.0.to_vec())
    }

    fn walk_tree(&self, _: &Path, _: &Pattern) -> Result<Vec<MetricResponseLeaf>, Error> {
        Ok(vec![MetricResponseLeaf {
            name: "virtual".to_owned(),
            path: "metric".to_owned(),
            is_leaf: true,
        }])
    }
}

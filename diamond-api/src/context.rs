use crate::opts::Args;
use crate::storage::Storage;
use std::sync::Arc;

#[derive(Clone)]
pub struct Context {
    pub args: Args,
    pub storage: Arc<dyn Storage + Send + Sync>,
}

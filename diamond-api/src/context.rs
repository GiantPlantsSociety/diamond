use crate::opts::*;
use crate::storage::Walker;

#[derive(Clone)]
pub struct Context<T: Walker> {
    pub args: Args,
    pub walker: T,
}

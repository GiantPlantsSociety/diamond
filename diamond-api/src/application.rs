use crate::find::*;
use crate::opts::*;
use crate::render::*;
use actix_web::web::{resource, ServiceConfig};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Context {
    pub args: Args,
    pub walker: Walker,
}

#[derive(Clone)]
pub enum Walker {
    File(PathBuf),
    Const(Vec<RenderPoint>),
}

pub fn app_config(ctx: Context) -> impl Fn(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        config
            .data(ctx.clone())
            .service(resource("/render").to_async(render_handler))
            .service(resource("/metrics/find").to_async(find_handler))
            .service(resource("/metrics").to_async(find_handler));
    }
}

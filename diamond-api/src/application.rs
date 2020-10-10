use actix_web::web::{resource, ServiceConfig};

use crate::context::Context;
use crate::find::*;
use crate::render::*;

pub fn app_config(ctx: Context) -> impl Fn(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        config
            .data(ctx.clone())
            .service(resource("/render").to(render_handler))
            .service(resource("/metrics/find").to(find_handler))
            .service(resource("/metrics").to(find_handler));
    }
}

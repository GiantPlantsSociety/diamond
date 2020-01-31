use actix_web::web::{resource, ServiceConfig};

use crate::context::Context;
use crate::find::*;
use crate::render::*;
use crate::storage::Walker;

pub fn app_config<T: 'static + Walker + Clone>(ctx: Context<T>) -> impl Fn(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        config
            .data(ctx.clone())
            .service(resource("/render").to(render_handler::<T>))
            .service(resource("/metrics/find").to(find_handler::<T>))
            .service(resource("/metrics").to(find_handler::<T>));
    }
}

use crate::find::*;
use crate::opts::*;
use crate::render::*;
use actix_web::web::{resource, ServiceConfig};

pub fn app_config(args: Args) -> impl Fn(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        config
            .data(args.clone())
            .service(resource("/render").to_async(render_handler))
            .service(resource("/metrics/find").to_async(find_handler))
            .service(resource("/metrics").to_async(find_handler));
    }
}

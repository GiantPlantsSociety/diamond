use crate::find::*;
use crate::opts::*;
use crate::render::*;
use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse};

pub fn create_app(opt: Args) -> App<Args> {
    App::with_state(opt)
        .middleware(Logger::default())
        .resource("/render", |r| r.with(render_handler))
        .resource("/metrics/find", |r| r.with(find_handler))
        .resource("/metrics", |r| r.with(find_handler))
        .default_resource(|_| HttpResponse::NotFound())
}

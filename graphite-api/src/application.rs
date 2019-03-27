use crate::find::*;
use crate::opts::*;
use actix_web::middleware::Logger;
use actix_web::{http, pred, App, HttpResponse};

pub fn create_app(opt: Args) -> App<Args> {
    App::with_state(opt)
        .middleware(Logger::default())
        .resource("/metrics/find", |r| {
            r.method(http::Method::GET).with(metrics_find_get);
            r.method(http::Method::POST)
                .filter(pred::Header("Content-Type", "application/json"))
                .with(metrics_find_json);
            r.method(http::Method::POST)
                .filter(pred::Header(
                    "Content-Type",
                    "application/www-form-urlencoded",
                ))
                .with(metrics_find_form)
        })
        .resource("/metrics", |r| {
            r.method(http::Method::GET).with(metrics_find_get);
            r.method(http::Method::POST)
                .filter(pred::Header("Content-Type", "application/json"))
                .with(metrics_find_json);
            r.method(http::Method::POST)
                .filter(pred::Header(
                    "Content-Type",
                    "application/www-form-urlencoded",
                ))
                .with(metrics_find_form)
        })
        .default_resource(|_| HttpResponse::NotFound())
}

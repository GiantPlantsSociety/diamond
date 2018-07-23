extern crate actix;
extern crate actix_web;
extern crate failure;
extern crate graphite_api;
extern crate structopt;

use actix_web::middleware::Logger;
use actix_web::server;
use graphite_api::opts::Args;
use graphite_api::*;
use std::fs::create_dir;
use std::process::exit;
use structopt::StructOpt;

fn main() {
    let args = Args::from_args();
    let path = args.path.clone();

    if !path.is_dir() {
        if args.force {
            eprintln!(
                "Directory {} is not found, trying to create it",
                path.display()
            );
            create_dir(&path).unwrap_or_else(|e| {
                eprintln!("{}", e);
                exit(1);
            });
            eprintln!("Directory {} has been created", path.display());
        } else {
            eprintln!("Directory {} is not found", path.display());
            exit(1);
        }
    }

    let sys = actix::System::new("graphite-api");
    server::new(move || create_app(args.clone()))
        .bind("127.0.0.1:8080")
        .unwrap()
        .start();

    let _ = sys.run();
}

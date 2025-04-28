use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use clap::Parser;
use diamond_api::application::app_config;
use diamond_api::context::Context;
use diamond_api::opts::Args;
use diamond_api::storage::whisper_fs::WhisperFileSystemStorage;
use std::fs::create_dir;
use std::io;
use std::process::exit;
use std::sync::Arc;

#[actix_web::main]
async fn run(args: Args) -> io::Result<()> {
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("RUST_LOG", "actix_web=info,actix_server=info") };
    env_logger::init();

    let path = &args.path;

    if !path.is_dir() {
        if args.force {
            eprintln!(
                "Directory {} is not found, trying to create it",
                path.display()
            );
            create_dir(path)?;
            eprintln!("Directory {} has been created", path.display());
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Directory {} is not found", path.display()),
            ));
        }
    }

    let listen = format!("127.0.0.1:{}", &args.port);

    let ctx = Context {
        storage: Arc::new(WhisperFileSystemStorage(args.path.clone())),
        args,
    };

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(app_config(ctx.clone()))
            .default_service(web::route().to(HttpResponse::NotFound))
    })
    .bind(listen)?
    .run()
    .await?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(args) {
        eprintln!("{}", err);
        exit(1);
    }
}

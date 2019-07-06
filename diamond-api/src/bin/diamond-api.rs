use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use env_logger;
use diamond_api::find::find_handler;
use diamond_api::opts::Args;
use diamond_api::render::render_handler;
use std::fs::create_dir;
use std::io;
use std::process::exit;
use structopt::StructOpt;

fn run(args: Args) -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
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

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(args.clone())
            .service(web::resource("/render").to_async(render_handler))
            .service(web::resource("/metrics/find").to_async(find_handler))
            .service(web::resource("/metrics").to_async(find_handler))
            .default_service(web::route().to(|| HttpResponse::NotFound()))
    })
    .bind(listen)?
    .run()?;

    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(args) {
        eprintln!("{}", err);
        exit(1);
    }
}

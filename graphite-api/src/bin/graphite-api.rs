use actix_web::server;
use env_logger;
use graphite_api::application::create_app;
use graphite_api::opts::Args;
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

    server::new(move || create_app(args.clone()))
        .bind(listen)?
        .run();

    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(args) {
        eprintln!("{}", err);
        exit(1);
    }
}

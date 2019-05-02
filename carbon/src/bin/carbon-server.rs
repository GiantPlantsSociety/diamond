extern crate tokio;

use carbon::line_update;
use carbon::settings::Settings;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use tokio::codec::Framed;
use tokio::codec::LinesCodec;
use tokio::net::TcpListener;
use tokio::prelude::*;
use std::process::exit;

#[derive(Debug, StructOpt)]
#[structopt(name = "carbon-server")]
struct Args {
    /// Path to config file
    #[structopt(name = "config", long = "config", short = "c", parse(from_os_str))]
    config: Option<PathBuf>,

    /// Generate default config file
    #[structopt(short = "-g", requires = "config")]
    generate: bool,
}

fn main() {
    let args = Args::from_args();

    if args.generate {
        Settings::generate(args.config.unwrap()).unwrap();
        exit(1);
    }

    let settings = Settings::new(args.config).expect("Failed to parse config");
    println!("{:?}", settings);


    let addr = format!("127.0.0.1:{}", settings.tcp.port).parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();
    let path = Arc::new(settings.db_path);

    let server = listener
        .incoming()
        .for_each(move |sock| {
            let framed_sock = Framed::new(sock, LinesCodec::new());
            let lpath = path.clone();
            framed_sock.for_each(move |line| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32;
                line_update(&line, &lpath, now).unwrap();
                Ok(())
            })
        })
        .map_err(|err| {
            println!("accept error = {:?}", err);
        });

    println!("server running on {}", addr);
    tokio::run(server);
}

extern crate tokio;

use carbon::line_update;
use carbon::settings::Settings;
use failure::Error;
use std::io;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use tokio::codec::Framed;
use tokio::codec::LinesCodec;
use tokio::net::{TcpListener, UdpFramed, UdpSocket};
use tokio::prelude::*;

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

fn run(args: Args) -> Result<(), Error> {
    if args.generate {
        Settings::generate(args.config.unwrap())?;
        exit(1);
    }

    let settings = Settings::new(args.config)?;

    let tcp_addr = format!("{0}:{1}", &settings.tcp.host, settings.tcp.port).parse()?;
    let udp_addr = format!("{0}:{1}", &settings.udp.host, settings.udp.port).parse()?;

    let tcp_listener = TcpListener::bind(&tcp_addr)?;
    let udp_listener = UdpSocket::bind(&udp_addr)?;

    let config_tcp = Arc::new(settings);
    let config_udp = config_tcp.clone();

    let server = tcp_listener
        .incoming()
        .for_each(move |sock| {
            let framed_sock = Framed::new(sock, LinesCodec::new());
            let conf = config_tcp.clone();

            framed_sock.for_each(move |line| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32;
                line_update(&line, &conf.db_path, &conf.whisper, now)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                Ok(())
            })
        })
        .map_err(|err| {
            eprintln!("accept error = {:?}", err);
        });

    println!("server running on tcp {}", tcp_addr);

    let udp_server = UdpFramed::new(udp_listener, LinesCodec::new())
        .for_each(move |(line, _)| {
            let conf = config_udp.clone();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
            line_update(&line, &conf.db_path, &conf.whisper, now)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            Ok(())
        })
        .map_err(|err| {
            eprintln!("accept error = {:?}", err);
        });

    println!("server running on udp {}", udp_addr);

    tokio::run({
        server
            .join(udp_server)
            .map(|_| ())
            .map_err(|e| println!("error = {:?}", e))
    });

    Ok(())
}

fn main() {
    let args = Args::from_args();
    if let Err(err) = run(args) {
        eprintln!("{}", err);
        exit(1);
    }
}

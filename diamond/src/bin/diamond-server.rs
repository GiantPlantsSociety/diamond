use diamond::settings::Settings;
use diamond::update_silently;
use failure::Error;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
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

    let tcp_server = tcp_listener
        .incoming()
        .for_each(move |sock| {
            let framed_sock = Framed::new(sock, LinesCodec::new());
            let conf = config_tcp.clone();

            framed_sock.for_each(move |line| {
                update_silently(&line, &conf);
                Ok(())
            })
        })
        .map_err(|err| {
            eprintln!("accept error = {:?}", err);
        });

    println!("server running on tcp {}", tcp_addr);

    let udp_server = UdpFramed::new(udp_listener, LinesCodec::new())
        .for_each(move |(line, _)| {
            update_silently(&line, &config_udp);
            Ok(())
        })
        .map_err(|err| {
            eprintln!("accept error = {:?}", err);
        });

    println!("server running on udp {}", udp_addr);

    tokio::run({
        tcp_server
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

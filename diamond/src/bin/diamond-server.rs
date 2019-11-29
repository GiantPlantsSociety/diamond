use diamond::settings::Settings;
use diamond::update_silently;
use futures::join;
use futures::stream::StreamExt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::net::{TcpListener, UdpSocket};
use tokio_util::codec::Framed;
use tokio_util::codec::LinesCodec;
use tokio_util::udp::UdpFramed;

#[derive(Debug, StructOpt)]
#[structopt(name = "diamond-server")]
struct Args {
    /// Path to config file
    #[structopt(name = "config", long = "config", short = "c", parse(from_os_str))]
    config: Option<PathBuf>,

    /// Generate default config file
    #[structopt(short = "-g", requires = "config")]
    generate: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    if args.generate {
        Settings::generate(args.config.unwrap())?;
        exit(1);
    }

    let settings = Settings::new(args.config)?;

    let tcp_addr: SocketAddr = format!("{0}:{1}", &settings.tcp.host, settings.tcp.port).parse()?;
    let udp_addr: SocketAddr = format!("{0}:{1}", &settings.udp.host, settings.udp.port).parse()?;

    let mut tcp_listener = TcpListener::bind(&tcp_addr).await?;
    println!("server running on tcp {}", tcp_addr);

    let udp_listener = UdpSocket::bind(&udp_addr).await?;
    println!("server running on udp {}", udp_addr);

    let config_tcp = Arc::new(settings);
    let config_udp = config_tcp.clone();

    let tcp_server = async move {
        let mut incoming = tcp_listener.incoming();
        while let Some(sock) = incoming.next().await {
            let mut framed_sock = Framed::new(sock.unwrap(), LinesCodec::new());
            let conf = config_tcp.clone();

            while let Some(line) = framed_sock.next().await {
                match line {
                    Ok(line) => update_silently(&line, &conf),
                    Err(e) => eprintln!("accept error = {:?}", e),
                }
            }
        }
    };

    let udp_server = async move {
        let mut incoming = UdpFramed::new(udp_listener, LinesCodec::new());
        while let Some(line) = incoming.next().await {
            match line {
                Ok((line, _)) => update_silently(&line, &config_udp),
                Err(e) => eprintln!("accept error = {:?}", e),
            }
        }
    };

    join!(udp_server, tcp_server);

    Ok(())
}

use clap::Parser;
use diamond::settings::Settings;
use diamond::update_silently;
use futures::join;
use futures::stream::StreamExt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use tokio::net::{TcpListener, UdpSocket};
use tokio_util::codec::Framed;
use tokio_util::codec::LinesCodec;
use tokio_util::udp::UdpFramed;

#[derive(Debug, clap::Parser)]
struct Args {
    /// Path to config file
    #[arg(name = "config", long = "config", short = 'c')]
    config: Option<PathBuf>,

    /// Generate default config file
    #[arg(short = 'g', requires = "config")]
    generate: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.generate {
        Settings::generate(args.config.unwrap())?;
        exit(1);
    }

    let settings = Settings::new(args.config)?;

    let tcp_addr: SocketAddr = format!("{0}:{1}", &settings.tcp.host, settings.tcp.port).parse()?;
    let udp_addr: SocketAddr = format!("{0}:{1}", &settings.udp.host, settings.udp.port).parse()?;

    let tcp_listener = TcpListener::bind(&tcp_addr).await?;
    println!("server running on tcp {}", tcp_addr);

    let udp_listener = UdpSocket::bind(&udp_addr).await?;
    println!("server running on udp {}", udp_addr);

    let config_tcp = Arc::new(settings);
    let config_udp = config_tcp.clone();

    let tcp_server = async move {
        loop {
            match tcp_listener.accept().await {
                Ok((sock, _)) => {
                    let local_config = config_tcp.clone();
                    tokio::spawn(async move {
                        let mut framed_sock = Framed::new(sock, LinesCodec::new());
                        while let Some(line) = framed_sock.next().await {
                            match line {
                                Ok(line) => update_silently(&line, &local_config),
                                Err(e) => eprintln!("tcp receive error = {:?}", e),
                            }
                        }
                    });
                }
                Err(e) => eprintln!("tcp accept error = {:?}", e),
            }
        }
    };

    let udp_server = async move {
        let mut incoming = UdpFramed::new(udp_listener, LinesCodec::new());
        while let Some(line) = incoming.next().await {
            let local_config = config_udp.clone();
            tokio::spawn(async move {
                match line {
                    Ok((line, _)) => update_silently(&line, &local_config),
                    Err(e) => eprintln!("udp receive error = {:?}", e),
                }
            });
        }
    };

    join!(udp_server, tcp_server);

    Ok(())
}

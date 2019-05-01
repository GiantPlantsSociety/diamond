extern crate tokio;

use carbon::line_update;
use carbon::settings::Settings;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::codec::Framed;
use tokio::codec::LinesCodec;
use tokio::net::TcpListener;
use tokio::prelude::*;

fn main() {
    let settings = Settings::new().expect("Failed to parse config");
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

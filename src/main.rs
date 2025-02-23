use core::config::main::Settings;
use core::tls::listener::create_tls_acceptor;
use std::convert::Infallible;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;
use std::str::FromStr;

use hyper::body::Bytes;
use http_body_util::Full;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioIo, TokioExecutor};
use tls_listener::TlsListener;
use tokio::net::TcpListener;

pub mod core;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    env_logger::init();

    let config = Settings::load();

    let addr = SocketAddrV4::new(Ipv4Addr::from_str(config.network.ip.as_str())?, config.network.port);

    let mut listener  = TlsListener::new(
        create_tls_acceptor(
            Path::new(config.network.tls.certificate.as_str()),
            Path::new(config.network.tls.private_key.as_str())
        ).unwrap(),
        TcpListener::bind(addr).await?);
    
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http2::Builder::new(TokioExecutor::new()).serve_connection(io, service_fn(hello)).await {
                eprint!("Error service connection: {:?}", err);
            }
        });
    }
}

async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {

    Ok(Response::new(Full::new(Bytes::from("Hello World!"))))
}
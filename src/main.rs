use core::config::main::Settings;
use core::errors::WebMQError;
use core::traits::AsyncStart;
use std::{error::Error, net::Ipv4Addr, str::FromStr};

use log::{debug, error};
use network::listener::http::HttpsListener;
use tls_listener::rustls::rustls;

pub mod core;
pub mod network;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();
    debug!("Initialized logger");
    let _ = rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls cryptography provider");
    debug!("Installed rustls cryptography provider.");

    let config = Settings::load();

    let Ok(ip) = Ipv4Addr::from_str(config.network.ip.as_str()) else {
        error!("Couldn't parse IP adress");
        return Err(WebMQError::Unrecoverable.into());
    };

    let mut listener: Box<dyn AsyncStart> = match HttpsListener::new(ip, config.network.port, config.network.tls).await {
        Ok(l) => Box::new(l),
        Err(e) => {
            error!("Couldn't create listener: {e}");
            return Err(WebMQError::Unrecoverable.into());
        }
    };

    listener.start().await;

    Ok(())
}

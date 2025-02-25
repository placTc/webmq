use std::{path::Path, sync::Arc};

use log::{error, info};
use tls_listener::rustls::{
    TlsAcceptor,
    rustls::pki_types::{CertificateDer, PrivateKeyDer},
};

use rustls::ServerConfig;

use crate::{core::errors::WebMQError, utils::file::get_file_buffer};

pub fn create_tls_acceptor(
    certificate_path: &Path,
    private_key_path: &Path,
) -> Result<TlsAcceptor, WebMQError> {
    let certificate = match load_certificate(certificate_path) {
        Ok(cert) => cert.into_owned(),
        Err(e) => {
            error!("{e}");
            return Err(e);
        }
    };
    let certificate_path = certificate_path.to_string_lossy();
    info!("Loaded certificate from {certificate_path}");

    let private_key = match load_private_key(private_key_path) {
        Ok(pkey) => pkey.clone_key(),
        Err(e) => {
            error!("{e}");
            return Err(e);
        }
    };
    let private_key_path = private_key_path.to_string_lossy();
    info!("Loaded private key from {private_key_path}");

    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], private_key);

    match tls_config {
        Ok(config) => Ok(Arc::new(config).into()),
        Err(e) => Err(WebMQError::TLS(format!(
            "Could not create TLS acceptor: {}",
            e
        ))),
    }
}

fn load_certificate(certificate_path: &Path) -> Result<CertificateDer, WebMQError> {
    match get_file_buffer(certificate_path).and_then(|buf| Ok(CertificateDer::from(buf))) {
        Ok(a) => Ok(a),
        Err(e) => Err(WebMQError::Config(format!(
            "Couldn't load certificate: {}",
            e.to_string()
        ))),
    }
}

fn load_private_key(private_key_path: &Path) -> Result<PrivateKeyDer, WebMQError> {
    match get_file_buffer(private_key_path).and_then(|buf| Ok(PrivateKeyDer::Pkcs1(buf.into()))) {
        Ok(a) => Ok(a),
        Err(e) => Err(WebMQError::Config(format!(
            "Couldn't load private key: {}",
            e.to_string()
        ))),
    }
}

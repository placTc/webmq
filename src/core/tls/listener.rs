use std::{path::Path, sync::Arc};

use tls_listener::rustls::{rustls::{pki_types::{CertificateDer, PrivateKeyDer}, ServerConfig}, TlsAcceptor};

use crate::{core::errors::WebMQError, utils::file::get_file_buffer};

pub fn create_tls_acceptor(certificate_path: &Path, private_key_path: &Path) -> Result<TlsAcceptor, WebMQError> {
    let private_key = load_private_key(private_key_path).unwrap().clone_key();
    let certificate =  load_certificate(certificate_path).unwrap().into_owned();
    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], private_key);

    match tls_config {
        Ok(config) => Ok(Arc::new(config).into()),
        Err(e) => Err(WebMQError::TLS(format!("Could not create TLS acceptor: {}", e)))
    }

}

fn load_certificate(certificate_path: &Path) -> Result<CertificateDer, WebMQError> {
    match get_file_buffer(certificate_path)
    .and_then(|buf| {
        Ok(CertificateDer::from(buf))
    }) {
        Ok(a) => Ok(a),
        Err(e) => Err(WebMQError::Config(format!("Couldn't load certificate: {}", e.to_string())))
    }
}

fn load_private_key(private_key_path: &Path) -> Result<PrivateKeyDer, WebMQError> {
    match get_file_buffer(private_key_path)
    .and_then(|buf| {
        Ok(PrivateKeyDer::Pkcs1(buf.into()))
    }) {
        Ok(a) => Ok(a),
        Err(e) => Err(WebMQError::Config(format!("Couldn't load private key: {}", e.to_string())))
    }
}
#[derive(Debug, serde::Deserialize)]
pub struct TlsSettings {
    #[serde(default = "TlsSettings::default_certificate")]
    pub certificate: String,
    #[serde(default = "TlsSettings::default_private_key")]
    pub private_key: String,
    #[serde(default = "TlsSettings::default_algorithm")]
    pub algorithm: String,
}

impl TlsSettings {
    fn default_certificate() -> String {
        "./certificate.crt".to_string()
    }

    fn default_private_key() -> String {
        "./key.pem".to_string()
    }

    fn default_algorithm() -> String {
        "RSA".to_string()
    }
}

impl Default for TlsSettings {
    fn default() -> Self {
        Self {
            certificate: Self::default_certificate(),
            private_key: Self::default_private_key(),
            algorithm: Self::default_algorithm(),
        }
    }
}

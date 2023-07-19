use std::sync::Arc;
use bevy_stardust_shared::rustls::{ServerConfig, Certificate, PrivateKey};

pub(super) fn server_config(
    certificates: Vec<Certificate>,
    private_key: PrivateKey,
) -> Arc<ServerConfig> {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certificates, private_key)
        .expect("Bad certificates/private key");

    Arc::new(config)
}
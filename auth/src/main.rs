pub mod config;
pub mod server;

use log::info;
use crate::config::config;


fn main() {
    // Read config file
    let config = config();

    // Set up logger
    let mut logger = env_logger::builder();
    if config.logging.verbose { logger.parse_filters("trace"); }
    logger.init();

    info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    // let config = Arc::new(ServerConfig::builder()
    //     .with_safe_defaults()
    //     .with_no_client_auth()
    //     .with_single_cert(cert_chain, key_der));
}
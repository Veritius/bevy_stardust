pub mod config;
pub mod server;
pub mod crypto;


use log::info;
use mio::Events;
use crate::{config::config, server::{setup_server, LISTENER}};


fn main() {
    // Read config file
    let config = config();

    // Set up logger
    let mut logger = env_logger::builder();
    if config.logging.verbose { logger.parse_filters("trace"); }
    logger.init();

    info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let (mut server, mut poll) = setup_server(&config);
    info!("Server set up, starting...");

    let mut events = Events::with_capacity(256);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                LISTENER => server.accept(poll.registry()).unwrap(),
                _        => server.connection_event(poll.registry(), event),
            }
        }
    }
}
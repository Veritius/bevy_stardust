use std::net::SocketAddr;

#[derive(Debug)]
pub(super) struct OutgoingConnectionAttempt {
    pub address: SocketAddr,
    pub reassign_to: Option<u16>,
}

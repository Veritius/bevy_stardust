use std::net::SocketAddr;

#[derive(Debug)]
pub(super) struct IncomingConnectionAttempt {
    pub address: SocketAddr,
    pub req_port: u16,
    pub new_port: Option<u16>,
}
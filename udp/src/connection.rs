use std::net::SocketAddr;

pub struct Connection(pub(crate) ConnectionInner);

pub(crate) struct ConnectionInner {
    pub address: SocketAddr,
}
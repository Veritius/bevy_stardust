use std::net::SocketAddr;

/// An existing connection.
pub(crate) struct Connection {
    pub address: SocketAddr,

    #[cfg(feature="encryption")]
    pub tls_data: TlsData,
}

#[cfg(feature="encryption")]
pub(crate) enum TlsData {
    None,
    Server(rustls::ServerConnection),
    Client(rustls::ClientConnection),
}
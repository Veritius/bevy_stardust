use std::{net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, sync::Arc};
use bevy::{ecs::system::SystemParam, prelude::*};
use quinn_proto::{EndpointConfig, ServerConfig};
use crate::{Endpoint, Connection};

/// A [`SystemParam`] that allows creating [`Endpoint`] entities and [`Connection`] entities.
#[derive(SystemParam)]
pub struct QuinnManager<'w, 's> {
    commands: Commands<'w, 's>
}

impl<'w, 's> QuinnManager<'w, 's> {
    /// Create a new Quinn endpoint.
    /// 
    /// If successful, returns the local address the endpoint is bound to.
    /// 
    /// ## Servers
    /// If `server_config` is `None`, the endpoint will not be able to act as a server.
    /// The server config can be added or replaced at any time by using [`set_server_config`](Endpoint::set_server_config).
    /// 
    /// Endpoints will always be able to act as a client, through the [`connect`](Endpoint::connect) method.
    /// 
    /// ## Binding
    /// If `bind_address` is `None`, the OS will automatically assign an address to the socket.
    /// This is useful for clients, which don't need to have a known IP/port, but can make servers
    /// unreachable, such as in cases where port forwarding is needed.
    /// 
    /// If there is already a socket at the given address, `Err` is returned.
    pub fn open_endpoint(
        &mut self,
        quic_config: Arc<EndpointConfig>,
        server_config: Option<Arc<ServerConfig>>,
        bind_address: Option<SocketAddr>,
    ) -> anyhow::Result<SocketAddr> {
        // Giving this address to the OS means it assigns one for us
        const UNSPECIFIED: SocketAddr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            0
        );

        // Bind and configure the socket
        let socket = UdpSocket::bind(bind_address.unwrap_or(UNSPECIFIED))?;
        socket.set_nonblocking(true)?;

        // Build the inner endpoint
        let quinn = quinn_proto::Endpoint::new(
            quic_config,
            server_config,
            true,
            None,
        );

        // Create the endpoint and fetch some data as we're going to lose ownership soon
        let endpoint = Endpoint::new_inner(socket, quinn);
        let address = endpoint.local_addr();

        // Spawn the endpoint as an entity
        let entity = self.commands.spawn(endpoint).id();

        // Return the address
        return Ok(address);
    }
}
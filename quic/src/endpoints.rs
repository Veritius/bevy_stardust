use std::{net::{SocketAddr, ToSocketAddrs, UdpSocket}, sync::{Arc, Exclusive}};
use anyhow::{Context, Result};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_stardust::connections::peer::NetworkPeer;
use quinn_proto::*;
use rustls::{Certificate, PrivateKey, RootCertStore};
use crate::{connections::QuicConnectionBundle, plugin::PluginConfig, QuicConnection};

/// An active QUIC endpoint.
#[derive(Component)]
pub struct QuicEndpoint {
    pub(crate) quic_endpoint: Exclusive<Endpoint>,
    pub(crate) udp_socket: UdpSocket,
    pub(crate) root_certs: Arc<RootCertStore>,

    close_requested: bool,
}

impl QuicEndpoint {
    /// Returns the local address that the endpoint is connected to.
    pub fn local_address(&self) -> SocketAddr {
        self.udp_socket.local_addr().unwrap()
    }

    /// Marks the endpoint for closing, disconnecting all clients and shutting down the connection.
    pub fn close(&mut self) {
        self.close_requested = true;
        self.quic_endpoint.get_mut().reject_new_connections();
    }

    /// Returns `true` if the endpoint is marked for closing.
    pub fn is_closing(&self) -> bool {
        self.close_requested
    }

    #[must_use]
    pub(crate) fn connect(&mut self, address: SocketAddr, server_name: &str) -> Result<(ConnectionHandle, Connection)> {
        let client_config = rustls::ClientConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_root_certificates(self.root_certs.clone())
            .with_no_client_auth();

        Ok(self.quic_endpoint.get_mut().connect(ClientConfig::new(Arc::new(client_config)), address, server_name)?)
    }
}

/// Tool for opening QUIC endpoints.
#[derive(SystemParam)]
pub struct QuicConnectionManager<'w, 's> {
    plugin_config: Res<'w, PluginConfig>,
    endpoints: Query<'w, 's, &'static mut QuicEndpoint>,
    commands: Commands<'w, 's>,
}

impl QuicConnectionManager<'_, '_> {
    /// Opens a client (outgoing-only) endpoint.
    pub fn open_client_endpoint(
        &mut self,
        address: impl ToSocketAddrs,
        root_certs: Arc<RootCertStore>,
    ) -> Result<Entity> {
        Ok(self.commands.spawn(QuicEndpoint {
            quic_endpoint: Endpoint::new(
                self.plugin_config.endpoint_config.clone(),
                None,
                true).into(),
            udp_socket: Self::try_open_socket(address)?,
            root_certs,
            close_requested: false,
        }).id())
    }

    /// Opens a server (outgoing or incoming) endpoint.
    pub fn open_server_endpoint(
        &mut self,
        address: impl ToSocketAddrs,
        root_certs: Arc<RootCertStore>,
        certificate_chain: Vec<Certificate>,
        private_key: PrivateKey,
    ) -> Result<Entity> {
        let crypto = rustls::ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_no_client_auth()
            .with_single_cert(certificate_chain, private_key)?;

        Ok(self.commands.spawn(QuicEndpoint {
            quic_endpoint: Endpoint::new(
                self.plugin_config.endpoint_config.clone(),
                Some(Arc::new(ServerConfig::with_crypto(Arc::new(crypto)))),
                true).into(),
            udp_socket: Self::try_open_socket(address)?,
            root_certs,
            close_requested: false,
        }).id())
    }

    /// Try to connect to a remote server.
    /// The connection will be established on the endpoint bound to the `local` address.
    /// 
    /// The first value provided by the `ToSocketAddr` implementation will be used.
    pub fn try_connect_remote(
        &mut self,
        endpoint: Entity,
        remote: impl ToSocketAddrs,
        server_name: &str,
    ) -> Result<Entity> {
        // Get a single SocketAddr from remote
        let remote = remote.to_socket_addrs()?.nth(0)
            .context("No SocketAddr provided")?;

        // Find component for endpoint
        let mut endpoint = self.endpoints.get_mut(endpoint)?;

        // Connect to target with endpoint
        let (handle, connection) = endpoint.connect(remote, server_name)?;

        // Spawn entity to hold Connection
        Ok(self.commands.spawn(QuicConnectionBundle {
            peer_comp: NetworkPeer::new(),
            quic_comp: QuicConnection {
                handle,
                inner: Exclusive::new(connection),
            },
        }).id())
    }

    fn try_open_socket(address: impl ToSocketAddrs) -> Result<UdpSocket> {
        let socket = UdpSocket::bind(address)?;
        socket.set_nonblocking(true)?;
        Ok(socket)
    }
}
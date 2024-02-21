use std::{net::{SocketAddr, ToSocketAddrs, UdpSocket}, sync::{Arc, Exclusive}, collections::HashMap};
use anyhow::{Context, Result};
use bevy_ecs::{prelude::*, system::SystemParam, entity::Entities};
use quinn_proto::*;
// UdpState is such a terrible type name I needed to rename it
use quinn_udp::{UdpSockRef, UdpSocketState, UdpState as UdpCapability};
use rustls::{Certificate, PrivateKey, RootCertStore};
use crate::{plugin::PluginConfig, QuicConnection};

/// An active QUIC endpoint.
#[derive(Component)]
pub struct QuicEndpoint {
    pub(crate) inner: Exclusive<Endpoint>,
    pub(crate) connections: HashMap<ConnectionHandle, Entity>,

    root_certs: Arc<RootCertStore>,

    udp_socket: UdpSocket,
    socket_state: UdpSocketState,
    socket_capabilities: UdpCapability,

    close_requested: bool,
}

impl QuicEndpoint {
    pub(crate) fn new(
        endpoint: Endpoint,
        root_certs: Arc<RootCertStore>,
        socket_addr: impl ToSocketAddrs,
    ) -> Result<Self> {
        let udp_socket = UdpSocket::bind(socket_addr)?;
        UdpSocketState::configure(UdpSockRef::from(&udp_socket))?;

        Ok(Self {
            inner: Exclusive::new(endpoint),
            connections: HashMap::new(),
            root_certs,
            udp_socket,
            socket_state: UdpSocketState::new(),
            socket_capabilities: UdpCapability::new(),
            close_requested: false,
        })
    }

    #[must_use]
    pub(crate) fn connect(
        &mut self,
        entities: &Entities,
        address: SocketAddr,
        server_name: &str,
        transport: Arc<TransportConfig>,
        verifier: Arc<dyn crate::crypto::ServerCertVerifier>
    ) -> Result<(Entity, ConnectionHandle, Connection)> {
        let crypto = Self::build_client_config(Arc::new(crate::crypto::ServerCertVerifierWrapper {
            roots: self.root_certs.clone(),
            inner: verifier,
        }))?;

        let mut client_config = ClientConfig::new(Arc::new(crypto));
        client_config.transport_config(transport);

        let (handle, connection) = self.inner.get_mut().connect(
            client_config,
            address,
            server_name
        )?;

        let id = entities.reserve_entity();
        self.connections.insert(handle, id);
        Ok((id, handle, connection))
    }

    pub(crate) fn recv_split_borrow(&mut self) -> (&mut Endpoint, &HashMap<ConnectionHandle, Entity>, &UdpSocket, &UdpSocketState) {
        (self.inner.get_mut(), &self.connections, &self.udp_socket, &self.socket_state)
    }

    pub(crate) fn send_split_borrow(&self) -> (&UdpSocket, &UdpSocketState, &UdpCapability) {
        (&self.udp_socket, &self.socket_state, &self.socket_capabilities)
    }

    /// Returns the local address that the endpoint is connected to.
    pub fn local_address(&self) -> SocketAddr {
        self.udp_socket.local_addr().unwrap()
    }

    /// Marks the endpoint for closing, disconnecting all clients and shutting down the connection.
    pub fn close(&mut self) {
        self.close_requested = true;
        self.inner.get_mut().reject_new_connections();
    }

    /// Returns `true` if the endpoint is marked for closing.
    pub fn is_closing(&self) -> bool {
        self.close_requested
    }

    fn build_client_config(verifier: Arc<dyn rustls::client::ServerCertVerifier>) -> Result<rustls::ClientConfig> {
        Ok(rustls::ClientConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth())
    }
}

/// Tool for opening QUIC endpoints.
#[derive(SystemParam)]
pub struct QuicConnectionManager<'w, 's> {
    plugin_config: Res<'w, PluginConfig>,
    endpoints: Query<'w, 's, &'static mut QuicEndpoint>,
    entities: &'w Entities,
    commands: Commands<'w, 's>,
}

impl QuicConnectionManager<'_, '_> {
    /// Opens a client (outgoing-only) endpoint.
    pub fn open_client_endpoint(
        &mut self,
        address: impl ToSocketAddrs,
        root_certs: Arc<RootCertStore>,
    ) -> Result<Entity> {
        let address = address.to_socket_addrs()?.nth(0)
            .context("No SocketAddr provided")?;

        let endpoint = Endpoint::new(
            self.plugin_config.endpoint_config.clone(),
            None,
            false
        );

        let id = self.commands.spawn(QuicEndpoint::new(
            endpoint,
            root_certs,
            address
        )?).id();

        tracing::info!("Opened client endpoint {id:?} on {address}");
        Ok(id)
    }

    /// Opens a server (outgoing or incoming) endpoint.
    pub fn open_server_endpoint(
        &mut self,
        address: impl ToSocketAddrs,
        root_certs: Arc<RootCertStore>,
        certificate_chain: Vec<Certificate>,
        private_key: PrivateKey,
    ) -> Result<Entity> {
        let address = address.to_socket_addrs()?.nth(0)
            .context("No SocketAddr provided")?;

        let crypto = rustls::ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_no_client_auth()
            .with_single_cert(certificate_chain, private_key)?;

        let mut config = ServerConfig::with_crypto(Arc::new(crypto));
        config.transport_config(self.plugin_config.transport_config.clone());

        let endpoint = Endpoint::new(
            self.plugin_config.endpoint_config.clone(),
            Some(Arc::new(config)),
            false
        );

        let id = self.commands.spawn(QuicEndpoint::new(
            endpoint,
            root_certs,
            address
        )?).id();

        tracing::info!("Opened server endpoint {id:?} on {address}");
        Ok(id)
    }

    /// Try to connect to a remote server.
    /// The connection will be established on the endpoint bound to the `local` address.
    /// 
    /// The first value provided by the `ToSocketAddr` implementation will be used.
    pub fn try_connect(
        &mut self,
        endpoint: Entity,
        remote: impl ToSocketAddrs,
        server_name: &str,
    ) -> Result<Entity> {
        // Get a single SocketAddr from remote
        let remote = remote.to_socket_addrs()?.nth(0)
            .context("No SocketAddr provided")?;

        // Find component for endpoint
        let mut endpoint_comp = self.endpoints.get_mut(endpoint)?;

        // Connect to target with endpoint
        let (entity, handle, connection) = endpoint_comp.connect(
            self.entities,
            remote.clone(),
            server_name,
            self.plugin_config.transport_config.clone(),
            self.plugin_config.server_cert_verifier.clone()
        )?;

        // Spawn entity to hold Connection
        self.commands.get_or_spawn(entity).insert(QuicConnection::new(endpoint, handle, connection));

        tracing::info!("Created new connection {entity:?} to remote peer {remote} on endpoint {endpoint:?}");
        Ok(entity)
    }

    /// Like [`try_connect`](Self::try_connect) but with a custom certificate verifier.
    #[cfg(feature="insecure")]
    pub fn try_connect_with_custom_verifier(
        &mut self,
        endpoint: Entity,
        remote: impl ToSocketAddrs,
        server_name: &str,
        verifier: Arc<dyn crate::crypto::ServerCertVerifier>,
    ) -> Result<Entity> {
        // Get a single SocketAddr from remote
        let remote = remote.to_socket_addrs()?.nth(0)
            .context("No SocketAddr provided")?;

        // Find component for endpoint
        let mut endpoint_comp = self.endpoints.get_mut(endpoint)?;

        // Connect to target with endpoint using custom verifier
        let (entity, handle, connection) = endpoint_comp.connect(
            self.entities,
            remote.clone(),
            server_name,
            self.plugin_config.transport_config.clone(),
            verifier
        )?;

        // Spawn entity to hold Connection
        self.commands.get_or_spawn(entity).insert(QuicConnection::new(endpoint, handle, connection));

        tracing::info!("Connecting to remote peer {remote} on endpoint {endpoint:?} with custom verifier");
        Ok(entity)
    }

    fn try_open_socket(address: impl ToSocketAddrs) -> Result<UdpSocket> {
        let socket = UdpSocket::bind(address)?;
        socket.set_nonblocking(true)?;
        Ok(socket)
    }
}
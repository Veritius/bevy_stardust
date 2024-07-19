use std::{marker::PhantomData, net::{IpAddr, ToSocketAddrs}};
use anyhow::{Context, Result};
use crate::{AppProtos, Credentials, TrustAnchors};
use super::*;

/// A type annotation indicating an endpoint that can act as both a [`Server`] and [`Client`].
/// Used by [`EndpointBuilder`].
pub enum Dual {}

/// A type annotation indicating a server.
/// Used by [`EndpointBuilder`].
pub enum Server {}

/// A type annotation indicating a client.
/// Used by [`EndpointBuilder`].
pub enum Client {}

/// A builder for an [`Endpoint`].
pub struct EndpointBuilder<Side, State = ()> {
    side: PhantomData<Side>,
    state: State,
}

impl EndpointBuilder<(), ()> {
    /// Create an `EndpointBuilder` that can act as both a client and server.
    pub fn dual() -> EndpointBuilder<Dual, WantsSocket> {
        EndpointBuilder {
            side: PhantomData,
            state: WantsSocket { _hidden: () },
        }
    }

    /// Create an `EndpointBuilder` for a server.
    pub fn server() -> EndpointBuilder<Server, WantsSocket> {
        EndpointBuilder {
            side: PhantomData,
            state: WantsSocket { _hidden: () },
        }
    }

    /// Create an `EndpointBuilder` for a client.
    pub fn client() -> EndpointBuilder<Client, WantsSocket> {
        EndpointBuilder {
            side: PhantomData,
            state: WantsSocket { _hidden: () },
        }
    }
}

pub struct WantsSocket {
    _hidden: ()
}

impl<Side> EndpointBuilder<Side, WantsSocket> {
    /// Bind to a specific address, but lets the OS choose the port for us.
    pub fn with_address(self, address: impl Into<IpAddr>) -> Result<EndpointBuilder<Side, WantsProtos>> {
        Self::with_address_and_port(self, address.into(), 0u16)
    }

    /// Binds to a specific address and port number, creating a new `UdpSocket`.
    pub fn with_address_and_port(self, address: impl Into<IpAddr>, port: impl Into<u16>) -> Result<EndpointBuilder<Side, WantsProtos>> {
        Self::with_socket_addr(self, SocketAddr::new(address.into(), port.into()))
    }

    /// Bind to `sock_addr`, creating a new `UdpSocket`.
    pub fn with_socket_addr(self, sock_addr: impl ToSocketAddrs) -> Result<EndpointBuilder<Side, WantsProtos>> {
        // Resolve address
        let address = sock_addr
            .to_socket_addrs().with_context(|| anyhow::anyhow!("Failed to get address for socket"))?
            .next().ok_or_else(|| anyhow::anyhow!("Must have at least one address"))?;

        // Bind the socket
        let socket = UdpSocket::bind(address)?;
        Self::with_socket(self, socket)
    }

    /// Use an existing `UdpSocket`.
    pub fn with_socket(self, socket: UdpSocket) -> Result<EndpointBuilder<Side, WantsProtos>> {
        // Socket configuration
        socket.set_nonblocking(true)?;

        // Return the socket
        return Ok(EndpointBuilder {
            side: PhantomData,
            state: WantsProtos {
                socket
            },
        });
    }
}

pub struct WantsProtos {
    socket: UdpSocket,
}

impl<Side> EndpointBuilder<Side, WantsProtos> {
    /// Use a pre-existing [`AppProtos`].
    pub fn with_protos(self, protos: AppProtos) -> EndpointBuilder<Side, WantsTrustAnchors> {
        return EndpointBuilder {
            side: PhantomData,
            state: WantsTrustAnchors {
                socket: self.state.socket,
                protos,
            },
        };
    }
}

pub struct WantsTrustAnchors {
    socket: UdpSocket,
    protos: AppProtos,
}

impl EndpointBuilder<Dual, WantsTrustAnchors> {
    /// Use a pre-existing [`TrustAnchors`] store.
    pub fn with_trust_anchors(self, anchors: TrustAnchors) -> EndpointBuilder<Dual, WantsCredentials> {
        return EndpointBuilder {
            side: PhantomData,
            state: WantsCredentials {
                socket: self.state.socket,
                protos: self.state.protos,
                anchors,
            },
        };
    }
}

impl EndpointBuilder<Server, WantsTrustAnchors> {
    /// Use a pre-existing [`TrustAnchors`] store.
    pub fn with_trust_anchors(self, anchors: TrustAnchors) -> EndpointBuilder<Server, WantsCredentials> {
        return EndpointBuilder {
            side: PhantomData,
            state: WantsCredentials {
                socket: self.state.socket,
                protos: self.state.protos,
                anchors,
            },
        };
    }
}

impl EndpointBuilder<Client, WantsTrustAnchors> {
    /// Use a pre-existing [`TrustAnchors`] store.
    pub fn with_trust_anchors(self, anchors: TrustAnchors) -> EndpointBuilder<Client, ClientReady> {
        return EndpointBuilder {
            side: PhantomData,
            state: ClientReady {
                shared: ReadyShared {
                    socket: self.state.socket,
                    protos: self.state.protos,
                    anchors,
                    _hidden: (),
                },
                join: JoinShared {
                    _hidden: (),
                },
            },
        };
    }
}

pub struct WantsCredentials {
    socket: UdpSocket,
    protos: AppProtos,
    anchors: TrustAnchors,
}

impl EndpointBuilder<Dual, WantsCredentials> {
    /// Use a pre-existing [`Credentials`] set.
    pub fn with_credentials(self, credentials: Credentials) -> EndpointBuilder<Dual, DualReady> {
        return EndpointBuilder {
            side: PhantomData,
            state: DualReady {
                shared: ReadyShared {
                    socket: self.state.socket,
                    protos: self.state.protos,
                    anchors: self.state.anchors,
                    _hidden: (),
                },
                host: HostShared {
                    credentials,
                    _hidden: (),
                },
                join: JoinShared {
                    _hidden: (),
                },
            },
        };
    }
}

impl EndpointBuilder<Server, WantsCredentials> {
    /// Use a pre-existing [`Credentials`] set.
    pub fn with_credentials(self, credentials: Credentials) -> EndpointBuilder<Server, ServerReady> {
        return EndpointBuilder {
            side: PhantomData,
            state: ServerReady {
                shared: ReadyShared {
                    socket: self.state.socket,
                    protos: self.state.protos,
                    anchors: self.state.anchors,
                    _hidden: (),
                },
                host: HostShared {
                    credentials,
                    _hidden: (),
                }
            },
        };
    }
}

pub(crate) struct ReadyShared {
    pub(crate) socket: UdpSocket,
    pub(crate) protos: AppProtos,
    pub(crate) anchors: TrustAnchors,
    _hidden: (),
}

pub(crate) struct HostShared {
    pub(crate) credentials: Credentials,
    _hidden: (),
}

pub(crate) struct JoinShared {
    _hidden: (),
}

impl EndpointBuilder<Dual, DualReady> {
    /// Attempts to build the endpoint.
    pub fn build(self) -> Result<EndpointShared> {
        #[cfg(feature="quiche")]
        return crate::quiche::build_dual(self.state);
    }
}

pub struct DualReady {
    pub(crate) shared: ReadyShared,
    pub(crate) host: HostShared,
    pub(crate) join: JoinShared,
}

impl EndpointBuilder<Server, ServerReady> {
    /// Attempts to build the endpoint.
    pub fn build(self) -> Result<EndpointShared> {
        #[cfg(feature="quiche")]
        return crate::quiche::build_server(self.state);
    }
}

pub struct ServerReady {
    pub(crate) shared: ReadyShared,
    pub(crate) host: HostShared,
}

impl EndpointBuilder<Client, ClientReady> {
    /// Attempts to build the endpoint.
    pub fn build(self) -> Result<EndpointShared> {
        #[cfg(feature="quiche")]
        return crate::quiche::build_client(self.state);
    }
}

pub struct ClientReady {
    pub(crate) shared: ReadyShared,
    pub(crate) join: JoinShared,
}
use std::{marker::PhantomData, net::ToSocketAddrs};
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

mod sealed {
    pub trait Side {}
    impl Side for super::Dual {}
    impl Side for super::Server {}
    impl Side for super::Client {}
}

/// A builder for an [`Endpoint`].
pub struct EndpointBuilder<Side, State>
where
    Side: sealed::Side,
{
    side: PhantomData<Side>,
    state: State,
}

impl<Side, State> EndpointBuilder<Side, State>
where
    Side: sealed::Side,
{
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

impl<Side> EndpointBuilder<Side, WantsSocket>
where
    Side: sealed::Side
{
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

    /// Bind to `address`, creating a new `UdpSocket`.
    pub fn with_address(self, address: impl ToSocketAddrs) -> Result<EndpointBuilder<Side, WantsProtos>> {
        // Resolve address
        let address = address
            .to_socket_addrs().with_context(|| anyhow::anyhow!("Failed to get address for socket"))?
            .next().ok_or_else(|| anyhow::anyhow!("Must have at least one address"))?;

        // Bind the socket
        let socket = UdpSocket::bind(address)?;
        Self::with_socket(self, socket)
    }
}

pub struct WantsProtos {
    socket: UdpSocket,
}

impl<Side> EndpointBuilder<Side, WantsProtos>
where
    Side: sealed::Side
{
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
                }
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
                },
                host: HostShared {
                    credentials,
                }
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
                },
                host: HostShared {
                    credentials,
                }
            },
        };
    }
}

struct ReadyShared {
    socket: UdpSocket,
    protos: AppProtos,
    anchors: TrustAnchors,
}

struct HostShared {
    credentials: Credentials,
}

impl EndpointBuilder<Dual, DualReady> {
    /// Attempts to build the endpoint.
    pub fn build(self) -> Result<Endpoint> {
        todo!()
    }
}

pub struct DualReady {
    shared: ReadyShared,
    host: HostShared,
}

impl EndpointBuilder<Server, ServerReady> {
    /// Attempts to build the endpoint.
    pub fn build(self) -> Result<Endpoint> {
        todo!()
    }
}

pub struct ServerReady {
    shared: ReadyShared,
    host: HostShared,
}

impl EndpointBuilder<Client, ClientReady> {
    /// Attempts to build the endpoint.
    pub fn build(self) -> Result<Endpoint> {
        todo!()
    }
}

pub struct ClientReady {
    shared: ReadyShared,
}
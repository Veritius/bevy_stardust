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
    pub trait Sealed {}
    impl Sealed for super::Dual {}
    impl Sealed for super::Server {}
    impl Sealed for super::Client {}
}

pub trait Side: sealed::Sealed {

}

impl Side for Dual {

}

impl Side for Server {

}

impl Side for Client {

}

/// A builder for an [`Endpoint`].
pub struct EndpointBuilder<S, State>
where
    S: Side,
{
    side: PhantomData<S>,
    state: State,
}

impl<S, State> EndpointBuilder<S, State>
where
    S: Side,
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

impl<S> EndpointBuilder<S, WantsSocket>
where
    S: Side
{
    /// Use an existing `UdpSocket`.
    pub fn with_socket(self, socket: UdpSocket) -> Result<EndpointBuilder<S, WantsProtos>> {
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
    pub fn with_address(self, address: impl ToSocketAddrs) -> Result<EndpointBuilder<S, WantsProtos>> {
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

impl<S> EndpointBuilder<S, WantsProtos>
where
    S: Side
{
    /// Use a pre-existing [`AppProtos`].
    pub fn with_protos(self, protos: AppProtos) -> EndpointBuilder<S, WantsTrustAnchors> {
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
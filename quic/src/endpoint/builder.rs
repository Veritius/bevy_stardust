use std::marker::PhantomData;
use anyhow::{Context, Result};

use super::*;

/// A type annotation indicating neither client nor server.
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
    state: State,
    side: PhantomData<Side>,
}

impl<Side, State> EndpointBuilder<Side, State>
where
    Side: sealed::Side,
{
    /// Create an `EndpointBuilder` that can act as both a client and server.
    pub fn dual() -> EndpointBuilder<Dual, WantsSocket> {
        EndpointBuilder {
            state: WantsSocket { _hidden: () },
            side: PhantomData,
        }
    }

    /// Create an `EndpointBuilder` for a server.
    pub fn server() -> EndpointBuilder<Server, WantsSocket> {
        EndpointBuilder {
            state: WantsSocket { _hidden: () },
            side: PhantomData,
        }
    }

    /// Create an `EndpointBuilder` for a client.
    pub fn client() -> EndpointBuilder<Client, WantsSocket> {
        EndpointBuilder {
            state: WantsSocket { _hidden: () },
            side: PhantomData,
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
    pub fn with_socket(mut self, socket: UdpSocket) -> Result<EndpointBuilder<Side, WantsProtos>> {
        // Socket must be nonblocking
        socket.set_nonblocking(true)?;

        // Return the socket
        return Ok(EndpointBuilder {
            state: WantsProtos {
                socket
            },
            side: PhantomData,
        });
    }

    /// Bind to `address`, creating a new `UdpSocket`.
    pub fn with_address(self, address: SocketAddr) -> Result<EndpointBuilder<Side, WantsProtos>> {
        let socket = UdpSocket::bind(address)?;
        Self::with_socket(self, socket)
    }
}

pub struct WantsProtos {
    socket: UdpSocket,
}
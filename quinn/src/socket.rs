use std::{io::Result as IoResult, mem::MaybeUninit, net::SocketAddr};

/// An abstraction over UDP sockets that can be used for I/O.
pub(crate) trait SyncUdpSocket {
    /// Returns the address the socket is bound to.
    fn addr(&self) -> SocketAddr;

    /// Returns a datagram if one has been received.
    /// 
    /// `scratch` must be filled with the datagram.
    /// The length of the scratch buffer is used 
    ///
    /// Has the following return cases:
    /// - `Ok(Some(x))` - Successfully received the entire datagram
    /// - `Ok(None)` - No packets were available to read
    /// - `Err(x)` - Receive failure, `x` is the stdlib error type
    fn recv(
        &mut self,
        scratch: &mut [MaybeUninit<u8>],
    ) -> IoResult<Option<Receive>>;

    /// Sends a datagram.
    /// 
    /// Has the following return cases:
    /// - `Ok(x)` - send success, `x` is the amount of bytes sent
    /// - `Err(x)` - send failure, `x` is the stdlib io error type
    fn send(
        &mut self,
        transmit: Transmit,
        scratch: &mut [MaybeUninit<u8>],
    ) -> IoResult<usize>;
}

#[derive(Debug, Clone)]
pub struct Receive {
    pub length: usize,
    pub address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct Transmit<'a> {
    pub payload: &'a [u8],
    pub address: SocketAddr,
}

pub struct QuicSocket {
    socket: socket2::Socket,
}

impl QuicSocket {
    pub fn new(
        address: SocketAddr,
    ) -> IoResult<Self> {
        return Ok(Self {
            socket: socket2::Socket::new(
                socket2::Domain::for_address(address),
                socket2::Type::DGRAM,
                Some(socket2::Protocol::UDP),
            )?,
        });
    }
}

impl SyncUdpSocket for QuicSocket {
    fn addr(&self) -> SocketAddr {
        let sockaddr = self.socket.local_addr().unwrap();
        return sockaddr.as_socket().unwrap();
    }

    fn recv(
        &mut self,
        scratch: &mut [MaybeUninit<u8>],
    ) -> IoResult<Option<Receive>> {
        todo!()
    }

    fn send(
        &mut self,
        transmit: Transmit,
        scratch: &mut [MaybeUninit<u8>],
    ) -> IoResult<usize> {
        todo!()
    }
}
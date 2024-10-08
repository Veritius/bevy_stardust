use std::{io::{IoSliceMut, Result as IoResult}, net::{SocketAddr, UdpSocket}};

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
        scratch: &mut [IoSliceMut],
    ) -> IoResult<Option<Receive>>;

    /// Sends a datagram.
    /// 
    /// Has the following return cases:
    /// - `Ok(x)` - send success, `x` is the amount of bytes sent
    /// - `Err(x)` - send failure, `x` is the stdlib io error type
    fn send(
        &mut self,
        meta: Transmit,
        scratch: &mut [IoSliceMut],
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

impl SyncUdpSocket for UdpSocket {
    fn addr(&self) -> SocketAddr {
        self.local_addr().unwrap()
    }

    fn recv(
        &mut self,
        scratch: &mut [IoSliceMut],
    ) -> IoResult<Option<Receive>> {
        todo!()
    }

    fn send(
        &mut self,
        transmit: Transmit,
        scratch: &mut [IoSliceMut],
    ) -> IoResult<usize> {
        todo!()
    }
}
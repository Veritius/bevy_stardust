use std::{net::{SocketAddr, UdpSocket}, io::Result as IoResult};

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
        scratch: &mut [u8],
    ) -> IoResult<Option<DatagramRecvMeta>>;

    /// Sends a datagram.
    /// 
    /// Has the following return cases:
    /// - `Ok(x)` - send success, `x` is the amount of bytes sent
    /// - `Err(x)` - send failure, `x` is the stdlib io error type
    fn send(
        &mut self,
        meta: DatagramSendMeta,
        scratch: &mut [u8],
        datagram: &[u8],
    ) -> IoResult<usize>;
}

#[derive(Debug, Clone)]
pub struct DatagramRecvMeta {
    pub address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct DatagramSendMeta {
    pub address: SocketAddr,
}

impl SyncUdpSocket for UdpSocket {
    fn addr(&self) -> SocketAddr {
        self.local_addr().unwrap()
    }

    fn recv(
        &mut self,
        scratch: &mut [u8],
    ) -> IoResult<Option<DatagramRecvMeta>> {
        todo!()
    }

    fn send(
        &mut self,
        meta: DatagramSendMeta,
        scratch: &mut [u8],
        datagram: &[u8],
    ) -> IoResult<usize> {
        todo!()
    }
}
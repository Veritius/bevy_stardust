use std::{io::Result as IoResult, mem::MaybeUninit, net::{SocketAddr, UdpSocket}};

/// An abstraction over UDP sockets that can be used for I/O.
pub(crate) unsafe trait SyncUdpSocket {
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
    ) -> IoResult<Option<DatagramRecvMeta>>;

    /// Sends a datagram.
    /// 
    /// Has the following return cases:
    /// - `Ok(x)` - send success, `x` is the amount of bytes sent
    /// - `Err(x)` - send failure, `x` is the stdlib io error type
    fn send(
        &mut self,
        meta: DatagramSendMeta,
        scratch: &mut [MaybeUninit<u8>],
        datagram: &[u8],
    ) -> IoResult<usize>;
}

#[derive(Debug, Clone)]
pub struct DatagramRecvMeta {
    pub length: usize,
    pub address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct DatagramSendMeta {
    pub address: SocketAddr,
}

unsafe impl SyncUdpSocket for UdpSocket {
    fn addr(&self) -> SocketAddr {
        self.local_addr().unwrap()
    }

    fn recv(
        &mut self,
        scratch: &mut [MaybeUninit<u8>],
    ) -> IoResult<Option<DatagramRecvMeta>> {
        todo!()
    }

    fn send(
        &mut self,
        meta: DatagramSendMeta,
        scratch: &mut [MaybeUninit<u8>],
        datagram: &[u8],
    ) -> IoResult<usize> {
        todo!()
    }
}
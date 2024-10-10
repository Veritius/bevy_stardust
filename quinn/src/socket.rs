use std::{io::{ErrorKind, Result as IoResult}, mem::MaybeUninit, net::{SocketAddr, ToSocketAddrs}};

/// An owned bound UDP socket.
pub(crate) unsafe trait BoundUdpSocket {
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
    /// 
    /// # Safety
    /// The `length` field in a returned `Receive` **must** be
    /// equal to or lesser than the initialised data in `scratch`.
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

/// A UDP socket specialised for use in the crate.
pub struct QuicSocket {
    socket: socket2::Socket,
}

impl QuicSocket {
    /// Binds a new [`QuicSocket`].
    pub fn new(
        address: impl ToSocketAddrs,
    ) -> IoResult<Self> {
        // Get the address we'll be binding the socket to
        let address = match address.to_socket_addrs()?.next() {
            Some(val) => val,
            None => return Err(std::io::Error::from(ErrorKind::InvalidInput)),
        };

        // Construct socket
        let socket = socket2::Socket::new(
            socket2::Domain::for_address(address),
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        )?;

        // Configure socket
        socket.set_nonblocking(true)?;
        socket.bind(&address.into())?;

        return Ok(Self {
            socket,
        });
    }
}

unsafe impl BoundUdpSocket for QuicSocket {
    fn addr(&self) -> SocketAddr {
        let sockaddr = self.socket.local_addr().unwrap();
        return sockaddr.as_socket().unwrap();
    }

    fn recv(
        &mut self,
        scratch: &mut [MaybeUninit<u8>],
    ) -> IoResult<Option<Receive>> {
        let (length, address) = match self.socket.recv_from(scratch) {
            // Types returned by socket2 require some fiddling to be usable
            Ok((len, addr)) => (len, addr.as_socket()
                .ok_or_else(|| std::io::Error::from(ErrorKind::InvalidInput))?),

            // The blocking error is turned into Ok(None)
            Err(err) if err.kind() == ErrorKind::WouldBlock => return Ok(None),

            // Any other error is... actually an error
            Err(err) => return Err(err),
        };

        // Return recv metadata
        return Ok(Some(Receive {
            length,
            address,
        }));
    }

    fn send(
        &mut self,
        transmit: Transmit,
    ) -> IoResult<usize> {
        self.socket.send_to(
            transmit.payload,
            &(transmit.address.into()),
        )
    }
}
use std::{collections::VecDeque, io::ErrorKind, net::{SocketAddr, ToSocketAddrs}, thread::JoinHandle};
use bytes::BytesMut;
use crossbeam_channel::{Receiver, Sender, TryRecvError};

/// A UDP socket and associated thread for handling I/O with the operating system.
pub(crate) struct Socket {
    pub dgram_rx: Receiver<DgramRecv>,
    pub dgram_tx: Sender<DgramSend>,
    address: SocketAddr,
    thread: JoinHandle<Result<(), std::io::Error>>,
}

impl Socket {
    pub fn new(addr: impl ToSocketAddrs) -> Result<Self, std::io::Error> {
        // mio tokens as consts so they can be changed easily
        const SKT_READABLE_OR_WRITABLE: mio::Token = mio::Token(0);

        // Bind UDP socket and configure it
        let socket = std::net::UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        let address = socket.local_addr().unwrap();

        // Put udp socket in mio's wrapper type
        let mut socket = mio::net::UdpSocket::from_std(socket);

        // Channels for inter-thread communication
        let (dgram_recv_tx, dgram_recv_rx) = crossbeam_channel::unbounded::<DgramRecv>();
        let (dgram_send_tx, dgram_send_rx) = crossbeam_channel::unbounded::<DgramSend>();

        // Set up mio's polling system
        let mut mio_poll = mio::Poll::new()?;
        let mut mio_events = mio::Events::with_capacity(32);
        mio_poll.registry().register(&mut socket, SKT_READABLE_OR_WRITABLE, mio::Interest::READABLE | mio::Interest::WRITABLE)?;

        // Start thread
        let thread = std::thread::spawn(move || {
            let mut blocked_sends: VecDeque<DgramSend> = VecDeque::with_capacity(1);

            loop {
                mio_poll.poll(&mut mio_events, None)?;

                'events: for event in mio_events.iter() {
                    match event.token() {
                        SKT_READABLE_OR_WRITABLE => {
                            // TODO: Allow configuring scratch size
                            let mut scratch = vec![0u8; 1472];

                            match socket.recv_from(&mut scratch[..]) {
                                Ok((length, address)) => match dgram_recv_tx.send(DgramRecv {
                                    origin: address,
                                    payload: {
                                        let mut buf = BytesMut::with_capacity(scratch.len());
                                        buf.copy_from_slice(&scratch[..length]);
                                        buf
                                    },
                                }) {
                                    Ok(_) => {},
                                    Err(_) => todo!(),
                                },

                                Err(e) if e.kind() == ErrorKind::WouldBlock => {}, // Do nothing

                                Err(e) => todo!(),
                            }

                            while let Some(dgram) = blocked_sends.pop_front() {
                                match socket.send_to(&dgram.payload, dgram.target) {
                                    Ok(_) => {}, // Success

                                    // If this occurs we put it back into the queue
                                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                        blocked_sends.push_back(dgram);
                                        continue 'events;
                                    }

                                    // An actual I/O error occurred
                                    Err(e) => return Err(e),
                                }
                            }

                            loop { match dgram_send_rx.try_recv() {
                                Ok(dgram) => match socket.send_to(&dgram.payload, dgram.target) {
                                    Ok(_) => {}, // Success

                                    // If this occurs we queue it for attempted sending later on
                                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                        blocked_sends.push_back(dgram);
                                        continue 'events;
                                    }
                                    
                                    // An actual I/O error occurred
                                    Err(e) => return Err(e),
                                },
    
                                Err(TryRecvError::Empty) => {
                                    continue 'events;
                                },
    
                                // If this occurs it means that the handle has been dropped
                                Err(TryRecvError::Disconnected) => { return Ok(()); }
                            } }

                        }

                        // Shouldn't happen
                        _ => unimplemented!(),
                    }
                }
            }
        });

        return Ok(Socket {
            dgram_rx: dgram_recv_rx,
            dgram_tx: dgram_send_tx,
            address,
            thread,
        });
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.address
    }
}

pub(crate) struct DgramRecv {
    pub origin: SocketAddr,
    pub payload: BytesMut,
}

pub(crate) struct DgramSend {
    pub target: SocketAddr,
    pub payload: BytesMut,
}
use std::{collections::VecDeque, io::ErrorKind, net::{SocketAddr, ToSocketAddrs}, thread::JoinHandle};
use bytes::BytesMut;
use crate::channels::mpsc;

/// A UDP socket and associated thread for handling I/O with the operating system.
pub(crate) struct Socket {
    pub dgram_rx: mpsc::Receiver<DgramRecv>,
    dgram_tx: crossbeam_channel::Sender<DgramSend>,
    tx_alerts: mio::Waker,
    address: SocketAddr,
    thread: JoinHandle<Result<(), std::io::Error>>,
}

impl Socket {
    pub fn new(addr: impl ToSocketAddrs) -> Result<Self, std::io::Error> {
        // mio tokens as consts so they can be changed easily
        const SKT_READABLE_OR_WRITABLE: mio::Token = mio::Token(0);
        const OUTGOING_APP_PACKETS_QUEUED: mio::Token = mio::Token(1);

        // Bind UDP socket and configure it
        let socket = std::net::UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        let address = socket.local_addr().unwrap();

        // Put udp socket in mio's wrapper type
        let mut socket = mio::net::UdpSocket::from_std(socket);

        // Channels for inter-thread communication
        let (dgram_recv_tx, dgram_recv_rx) = mpsc::channel::<DgramRecv>();
        let (dgram_send_tx, dgram_send_rx) = crossbeam_channel::unbounded::<DgramSend>();

        // Set up mio's polling system
        let mut mio_poll = mio::Poll::new()?;
        let mut mio_events = mio::Events::with_capacity(32);
        mio_poll.registry().register(&mut socket, SKT_READABLE_OR_WRITABLE, mio::Interest::READABLE | mio::Interest::WRITABLE)?;
        let alerter = mio::Waker::new(mio_poll.registry(), OUTGOING_APP_PACKETS_QUEUED).unwrap();

        // Start thread
        let thread = std::thread::spawn(move || {
            let mut waiting_outgoing: VecDeque<DgramSend> = VecDeque::with_capacity(1);

            fn send_waiting_outgoing(
                socket: &mio::net::UdpSocket,
                queue: &mut VecDeque<DgramSend>,
            ) -> Result<(), std::io::Error> {
                while let Some(dgram) = queue.pop_front() {
                    match socket.send_to(&dgram.payload, dgram.target) {
                        Ok(_) => {
                            log::trace!("Sent datagram to {}: {:?}", dgram.target, &dgram.payload);
                            continue;
                        },

                        Err(e) => {
                            queue.push_front(dgram);
                            return Err(e);
                        },
                    };
                }

                return Ok(());
            }

            loop {
                mio_poll.poll(&mut mio_events, None)?;

                for event in mio_events.iter() {
                    match event.token() {
                        SKT_READABLE_OR_WRITABLE => {
                            // TODO: Allow configuring scratch size
                            let mut scratch = vec![0u8; 1472];

                            // The socket might be readable
                            match socket.recv_from(&mut scratch[..]) {
                                Ok((length, address)) => {
                                    log::trace!("Received datagram from {}: {:?}", address, &scratch[..length]);

                                    match dgram_recv_tx.send(DgramRecv {
                                        origin: address,
                                        payload: {
                                            let mut buf = BytesMut::with_capacity(scratch.len());
                                            buf.copy_from_slice(&scratch[..length]);
                                            buf
                                        },
                                    }) {
                                        Ok(_) => {},
                                        Err(_) => todo!(),
                                    }
                                },

                                Err(e) if e.kind() == ErrorKind::WouldBlock => {}, // Do nothing

                                Err(e) => todo!(),
                            }

                            // The socket might be writable
                            while let Some(dgram) = waiting_outgoing.pop_front() {
                                match socket.send_to(&dgram.payload, dgram.target) {
                                    Ok(_) => {
                                        log::trace!("Sent datagram to {}: {:?}", dgram.target, &dgram.payload);
                                    }, // Success

                                    // If this occurs we put it back into the queue
                                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                        waiting_outgoing.push_back(dgram);
                                        break; // sending is blocked
                                    }

                                    // An actual I/O error occurred
                                    Err(e) => return Err(e),
                                }
                            }
                        },

                        OUTGOING_APP_PACKETS_QUEUED => {
                            while let Ok(message) = dgram_send_rx.try_recv() {
                                waiting_outgoing.push_back(message);
                            }

                            match send_waiting_outgoing(&socket, &mut waiting_outgoing) {
                                // Success
                                Ok(()) => {},

                                // If it would block, do nothing, that's normal.
                                Err(e) if e.kind() == ErrorKind::WouldBlock => {},

                                // Any other error is abnormal
                                Err(e) => todo!(),
                            }
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
            tx_alerts: alerter,
            address,
            thread,
        });
    }

    pub fn send(&self, dgram: DgramSend) {
        log::trace!("Attempting to queue datagram for sending");
        self.dgram_tx.send(dgram);
        self.tx_alerts.wake();
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
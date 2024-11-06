use std::{net::SocketAddr, time::Duration};
use bevy_tasks::{IoTaskPool, Task};
use bytes::Bytes;
use crossbeam_channel::Sender;
use mio::{net::UdpSocket, Events, Interest, Poll, Token};

pub(super) struct AsyncUdpSocket(Task<()>);

impl AsyncUdpSocket {
    pub fn new(
        mut socket: UdpSocket,
        sender: Sender<Receive>,
    ) -> Self {
        return Self(IoTaskPool::get().spawn(async move {
            let mut poll = Poll::new().unwrap();
            let mut events = Events::with_capacity(128);
            
            poll.registry().register(
                &mut socket,
                Token(0),
                Interest::READABLE | Interest::WRITABLE
            ).unwrap();

            let mut scratch = vec![0u8; 1472]; // TODO: Make configurable

            loop {
                poll.poll(&mut events, Some(Duration::ZERO)).unwrap();

                for _event in events.iter() {
                    loop {
                        match socket.recv_from(&mut scratch) {
                            Ok((length, address)) => {
                                sender.send(Receive {
                                    address,
                                    payload: Bytes::copy_from_slice(&scratch[..length]),
                                }).unwrap(); // TODO: Handle errors
                            },

                            Err(ref err) if would_block(err) => break,
                            Err(_err) => return (), // TODO: Handle errors properly
                        }
                    }
                }
            }
        }));
    }

    // pub fn send(
    //     &self,
    //     transmit: Transmit,
    // ) {
    //     self.
    // }
}

fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}

pub(super) struct Receive {
    pub address: SocketAddr,
    pub payload: Bytes,
}

pub(super) struct Transmit<'a> {
    pub address: SocketAddr,
    pub payload: &'a [u8],
}
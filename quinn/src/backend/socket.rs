use std::{net::SocketAddr, sync::Arc, time::Duration};
use bevy_tasks::{IoTaskPool, Task};
use bytes::BytesMut;
use crossbeam_channel::Sender;
use mio::{net::UdpSocket, Events, Interest, Poll, Token};

pub(super) struct AsyncUdpSocket {
    socket: Arc<UdpSocket>,
    task: Task<()>,
}

impl AsyncUdpSocket {
    pub fn new(
        mut socket: UdpSocket,
        datagrams: Sender<Receive>,
    ) -> Self {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);
        
        poll.registry().register(
            &mut socket,
            Token(0),
            Interest::READABLE | Interest::WRITABLE,
        ).unwrap();

        let ret_socket = Arc::new(socket);
        let task_socket = ret_socket.clone();

        let task = IoTaskPool::get().spawn(async move {
            let mut scratch = vec![0u8; 1472]; // TODO: Make configurable

            loop {
                poll.poll(&mut events, Some(Duration::ZERO)).unwrap();

                for _event in events.iter() {
                    loop {
                        match task_socket.recv_from(&mut scratch) {
                            Ok((length, address)) => {
                                let mut payload = BytesMut::with_capacity(length);
                                payload.extend_from_slice(&scratch[..length]);
                                // TODO: Handle errors
                                datagrams.send(Receive { address, payload }).unwrap();
                            },
                            Err(ref err) if would_block(err) => break,
                            Err(_err) => return (), // TODO: Handle errors properly
                        }
                    }
                }
            }
        });

        return Self {
            socket: ret_socket,
            task,
        };
    }

    pub fn send(
        &self,
        transmit: Transmit,
    ) {
        self.socket.send_to(
            transmit.payload,
            transmit.address,
        ).unwrap(); // TODO: Handle errors
    }
}

fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}

pub(super) struct Receive {
    pub address: SocketAddr,
    pub payload: BytesMut,
}

pub(super) struct Transmit<'a> {
    pub address: SocketAddr,
    pub payload: &'a [u8],
}
use std::{io, net::{SocketAddr, UdpSocket}, pin::{pin, Pin}, sync::Arc, task::{Context, Poll}};
use async_io::{Async, Readable};
use bytes::BytesMut;
use futures_lite::{FutureExt, Stream, StreamExt};
use super::taskpool::get_task_pool;

pub(crate) struct SocketConfig {
    pub recv_buf_size: usize,
}

pub(super) struct DgramRecv {
    pub address: SocketAddr,
    pub payload: BytesMut,
}

pub(super) struct DgramSend {
    pub address: SocketAddr,
    pub payload: BytesMut,
}

pub(super) struct Socket {
    socket: Arc<Async<UdpSocket>>,
    task: async_task::Task<Result<(), io::Error>>,

    pub recv_rx: async_channel::Receiver<DgramRecv>,
    pub send_tx: async_channel::Sender<DgramSend>,
}

impl Socket {
    pub fn new(
        socket: Arc<Async<UdpSocket>>,
        config: SocketConfig,
    ) -> Socket {
        let (recv_tx, recv_rx) = async_channel::unbounded();
        let (send_tx, send_rx) = async_channel::unbounded();

        let task = get_task_pool().spawn(driver(
            config,
            socket.clone(),
            recv_tx,
            send_rx,
        ));

        Socket {
            socket,
            task,
            recv_rx,
            send_tx,
        }
    }
}

async fn driver(
    config: SocketConfig,
    socket: Arc<Async<UdpSocket>>,
    recv_tx: async_channel::Sender<DgramRecv>,
    send_rx: async_channel::Receiver<DgramSend>,
) -> Result<(), io::Error> {
    let mut scratch: Vec<u8> = Vec::with_capacity(config.recv_buf_size);

    enum Event {
        Recv(DgramRecv),
        Send(DgramSend),

        IoError(io::Error),
    }

    let mut stream = pin!({
        // Stream for receiving datagrams from the socket
        let recv_stream = DgramRecvStream::new(&mut scratch[..], &socket)
        .map(|val| match val {
            Ok(val) => Event::Recv(val),
            Err(err) => Event::IoError(err),
        });

        // Stream for sending datagrams from the socket
        let send_stream = send_rx
            .map(|v| Event::Send(v));

        futures_lite::stream::or(
            recv_stream,
            send_stream,
        )
    });

    // Drain events as much as possible
    while let Some(event) = stream.next().await {
        match event {
            // Datagram received from socket
            Event::Recv(dgram_recv) => {
                // Add the datagram to the queue of channels
                if let Err(async_channel::SendError(_)) = recv_tx.send(dgram_recv).await {
                    // If an error occurs it means the channel is closed, so there's no receiver,
                    // so we just drop it since it'll never reach its intended target.
                    // We also return since there's no point keeping ourselves open now.
                    return Ok(());
                };
            },

            // Datagram queued for sending by someone
            Event::Send(dgram_send) => {
                match socket.send_to(&dgram_send.payload, dgram_send.address).await {
                    // Nothing of note
                    Ok(_) => { continue },

                    // IO error occurred when trying to send
                    Err(err) => return Err(err),
                }
            },

            // IO error reported by a stream or something
            Event::IoError(err) => return Err(err),
        }
    }

    todo!()
}

struct DgramRecvStream<'a> {
    scratch: &'a mut [u8],
    socket: &'a Async<UdpSocket>,
    readable: Readable<'a, UdpSocket>,
}

impl<'a> DgramRecvStream<'a> {
    fn new(
        scratch: &'a mut [u8],
        socket: &'a Async<UdpSocket>
    ) -> DgramRecvStream<'a> {
        DgramRecvStream {
            scratch,
            socket,
            readable: socket.readable(),
        }
    }
}

impl<'a> Stream for DgramRecvStream<'a> {
    type Item = Result<DgramRecv, io::Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.readable.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => {
                // This won't block as the readable handle is ready, which only returns ready when a read operation wouldn't block
                let r = futures_lite::future::block_on(self.socket.recv_from(&mut self.scratch))
                    .map(|(len, address)| DgramRecv {
                        address,
                        payload: (&self.scratch[..len]).into(),
                    });

                // Create a new Readable future so we can be woken
                // It's not stated what happens if it's polled after it's used
                // so we err on the side of caution and just recreate it
                self.readable = self.socket.readable();

                // Return our result
                return Poll::Ready(Some(r));
            },
        }
    }
}
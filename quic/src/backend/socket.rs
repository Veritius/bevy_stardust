use std::{io, net::{SocketAddr, UdpSocket}, pin::Pin, sync::Arc, task::{Context, Poll}};
use async_io::{Async, Readable};
use bytes::BytesMut;
use futures_lite::{FutureExt, Stream};
use super::taskpool::get_task_pool;

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
    ) -> Socket {
        let (recv_tx, recv_rx) = async_channel::unbounded();
        let (send_tx, send_rx) = async_channel::unbounded();

        let task = get_task_pool().spawn(driver(
            1472, // TODO: Make configurable.
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
    scratch: usize,
    socket: Arc<Async<UdpSocket>>,
    recv_tx: async_channel::Sender<DgramRecv>,
    send_rx: async_channel::Receiver<DgramSend>,
) -> Result<(), io::Error> {
    let mut scratch: Vec<u8> = Vec::with_capacity(scratch);

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

                // Reset the read future
                // It's not stated what happens if it's polled after it's used
                // so we err on the side of caution and just recreate it
                self.readable = self.socket.readable();

                // Return our result
                return Poll::Ready(Some(r));
            },
        }
    }
}
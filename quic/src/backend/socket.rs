use std::{io, net::{SocketAddr, UdpSocket}, sync::Arc};
use async_io::Async;
use bytes::BytesMut;
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
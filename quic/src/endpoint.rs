use std::{net::{ToSocketAddrs, UdpSocket}, sync::Arc};
use async_channel::{Receiver, Sender};
use async_io::Async;
use async_task::Task;
use crate::taskpool::get_task_pool;

/// A clonable handle to an endpoint.
#[derive(Clone)]
pub struct Endpoint(Arc<EndpointInner>);

impl Endpoint {
    /// Constructs a new [`Endpoint`].
    pub fn new<A>(
        address: A,
    ) -> Task<Result<Endpoint, EndpointError>>
    where
        A: ToSocketAddrs,
        A: Send + Sync + 'static,
        A::Iter: Send + Sync + 'static,
    {
        return get_task_pool().spawn(async move {
            // We have to bind the socket manually with blocking because AsyncToSocketAddrs has weird trait requirements
            // that can never be fulfilled while trying to use simple async closures like this. Oh well, it's good enough.
            let socket = blocking::unblock(move || Async::new(UdpSocket::bind(address)?)).await?;

            todo!()
        });
    }
}

/// An error returned during the creation or execution of an [`Endpoint`].
#[derive(Debug)]
pub enum EndpointError {
    /// An I/O error occurred.
    IoError(std::io::Error),

    /// A TLS error occurred.
    TlsError(rustls::Error),
}

impl From<std::io::Error> for EndpointError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

struct EndpointInner {
    io_socket: Arc<UdpSocket>,
    io_task: Task<Result<(), std::io::Error>>,

    io_recv_rx: Receiver<DgramRecv>,
    io_send_tx: Sender<DgramSend>,
}

async fn io_task(
    socket: Arc<UdpSocket>,
    io_recv_tx: Sender<DgramRecv>,
    io_send_rx: Receiver<DgramSend>,
) {
    todo!()
}

struct DgramRecv {

}

struct DgramSend {

}
use std::sync::Arc;
use async_channel::{Receiver, Sender};
use async_net::UdpSocket;
use async_task::Task;

/// A clonable handle to an endpoint.
#[derive(Clone)]
pub struct Endpoint(Arc<EndpointInner>);

impl Endpoint {
    /// Constructs a new [`Endpoint`].
    pub fn new(

    ) -> Task<Result<Endpoint, EndpointError>> {
        todo!()
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
use std::{net::SocketAddr, sync::Arc};
use quinn_proto::ConnectionHandle;
use crate::config::ClientConfig;
use super::{events::{C2EEvent, E2CEvent}, socket::DgramSend};

pub(super) struct OutgoingRequestParams {
    pub server_name: Arc<str>,
    pub remote_addr: SocketAddr,
    pub config: ClientConfig,
}

pub(super) fn new(
    params: OutgoingRequestParams,
) -> (
    OutgoingConnectionRequest,
    OutgoingConnectionReceiver,
) {
    let (tx, rx) = async_channel::bounded(1);

    let request = OutgoingConnectionRequest {
        tx: RequestNeedsResponse { tx },
        params,
    };

    let reciever = OutgoingConnectionReceiver { rx };

    return (request, reciever);
}

#[must_use]
pub(super) struct OutgoingConnectionRequest {
    params: OutgoingRequestParams,
    tx: RequestNeedsResponse,
}

impl OutgoingConnectionRequest {
    pub fn split(self) -> (
        OutgoingRequestParams,
        RequestNeedsResponse,
    ) {
        (
            self.params,
            self.tx,
        )
    }
}

#[must_use]
pub(super) struct RequestNeedsResponse {
    tx: async_channel::Sender<Response>,
}

impl RequestNeedsResponse {
    pub fn accept(self, data: impl Into<AcceptedData>) {
        let _ = self.tx.send_blocking(Response::Accepted(data.into()));
    }

    pub fn reject(self, data: impl Into<RejectedData>) {
        let _ = self.tx.send_blocking(Response::Rejected(data.into()));
    }
}

pub(super) struct OutgoingConnectionReceiver {
    pub rx: async_channel::Receiver<Response>,
}

pub(super) enum Response {
    Accepted(AcceptedData),
    Rejected(RejectedData),
}

pub(super) struct AcceptedData {
    pub quinn: quinn_proto::Connection,
    pub handle: quinn_proto::ConnectionHandle,

    pub e2c_rx: async_channel::Receiver<E2CEvent>,
    pub c2e_tx: async_channel::Sender<(ConnectionHandle, C2EEvent)>,

    pub dgram_tx: async_channel::Sender<DgramSend>,
}

pub(super) struct RejectedData {

}
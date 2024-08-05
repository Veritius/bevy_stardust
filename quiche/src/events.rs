use bytes::Bytes;
use crossbeam_channel::{unbounded, Receiver, Sender, TrySendError};

pub(crate) fn event_pair() -> (ConnectionEvents, EndpointEvents) {
    let (end_send, end_recv) = unbounded();
    let (con_send, con_recv) = unbounded();

    return (
        ConnectionEvents {
            recv: end_recv,
            send: con_send,
        },

        EndpointEvents {
            recv: con_recv,
            send: end_send,
        },
    )
}

/// A connection's handles to events.
pub(crate) struct ConnectionEvents {
    recv: Receiver<EndpointEvent>,
    send: Sender<ConnectionEvent>,
}

impl ConnectionEvents {
    pub fn try_send(&mut self, event: ConnectionEvent) -> Result<(), TrySendError<ConnectionEvent>> {
        self.send.try_send(event)
    }
}

/// An endpoint's handles to events.
pub(crate) struct EndpointEvents {
    recv: Receiver<ConnectionEvent>,
    send: Sender<EndpointEvent>,
}

impl EndpointEvents {
    pub fn try_send(&mut self, event: EndpointEvent) -> Result<(), TrySendError<EndpointEvent>> {
        self.send.try_send(event)
    }

    pub fn try_send_payload(&mut self, slice: &[u8]) -> Result<(), TrySendError<EndpointEvent>> {
        let payload = Bytes::copy_from_slice(slice);
        self.try_send(EndpointEvent::SendPacket { payload })
    }
}

/// Endpoint to connection directed event.
pub(crate) enum ConnectionEvent {
    Closed,

    RecvPacket {
        payload: Bytes,
    },
}

/// Connection to endpoint directed event.
pub(crate) enum EndpointEvent {
    Closed,

    SendPacket {
        payload: Bytes,
    },
}
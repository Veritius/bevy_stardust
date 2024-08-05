use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError, TrySendError};

pub(crate) fn event_pair() -> (ConnectionEvents, EndpointEvents) {
    let (end_send, end_recv) = unbounded();
    let (con_send, con_recv) = unbounded();

    return (
        ConnectionEvents {
            recv: con_recv,
            send: end_send,
        },

        EndpointEvents {
            recv: end_recv,
            send: con_send,
        },
    )
}

/// A connection's handles to events.
pub(crate) struct ConnectionEvents {
    recv: Receiver<ConnectionEvent>,
    send: Sender<EndpointEvent>,
}

impl ConnectionEvents {
    pub fn try_send(&mut self, event: EndpointEvent) -> Result<(), TrySendError<EndpointEvent>> {
        self.send.try_send(event)
    }

    pub fn try_recv(&mut self) -> Result<Option<ConnectionEvent>, TryRecvError> {
        match self.recv.try_recv() {
            Ok(event) => Ok(Some(event)),

            Err(TryRecvError::Empty) => Ok(None),

            Err(e) => Err(e),
        }
    }
}

/// An endpoint's handles to events.
pub(crate) struct EndpointEvents {
    recv: Receiver<EndpointEvent>,
    send: Sender<ConnectionEvent>,
}

impl EndpointEvents {
    pub fn try_send(&mut self, event: ConnectionEvent) -> Result<(), TrySendError<ConnectionEvent>> {
        self.send.try_send(event)
    }

    pub fn try_send_payload(&mut self, slice: &[u8]) -> Result<(), TrySendError<ConnectionEvent>> {
        self.try_send(ConnectionEvent::RecvPacket { payload: slice.into() })
    }

    pub fn try_recv(&mut self) -> Result<Option<EndpointEvent>, TryRecvError> {
        match self.recv.try_recv() {
            Ok(event) => Ok(Some(event)),

            Err(TryRecvError::Empty) => Ok(None),

            Err(e) => Err(e),
        }
    }
}

/// Endpoint to connection directed event.
pub(crate) enum ConnectionEvent {
    Closed,

    RecvPacket {
        payload: Box<[u8]>,
    },
}

/// Connection to endpoint directed event.
pub(crate) enum EndpointEvent {
    Closed,

    SendPacket {
        payload: Box<[u8]>,
    },
}
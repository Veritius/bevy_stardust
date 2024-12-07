use async_channel::{SendError, Sender};
use quinn_proto::ConnectionHandle;

pub(crate) enum E2CEvent {
    Quinn(quinn_proto::ConnectionEvent),
}

impl From<quinn_proto::ConnectionEvent> for E2CEvent {
    fn from(value: quinn_proto::ConnectionEvent) -> Self {
        Self::Quinn(value)
    }
}

pub(crate) enum C2EEvent {
    Quinn(quinn_proto::EndpointEvent),
}

impl From<quinn_proto::EndpointEvent> for C2EEvent {
    fn from(value: quinn_proto::EndpointEvent) -> Self {
        Self::Quinn(value)
    }
}

pub(crate) struct C2EEventSender {
    handle: ConnectionHandle,
    sender: Sender<(ConnectionHandle, C2EEvent)>,
}

impl C2EEventSender {
    pub fn new(
        handle: ConnectionHandle,
        sender: Sender<(ConnectionHandle, C2EEvent)>,
    ) -> C2EEventSender {
        C2EEventSender { handle, sender }
    }

    pub fn send(&self, event: C2EEvent) -> async_channel::Send<'_, (ConnectionHandle, C2EEvent)> {
        self.sender.send((
            self.handle,
            event
        ))
    }

    pub fn send_blocking(&self, event: C2EEvent) -> Result<(), SendError<C2EEvent>> {
        self.sender.send_blocking((
            self.handle,
            event
        )).map_err(|a| SendError(a.0.1))
    }
}
use bevy_stardust::prelude::ChannelMessage;
use bytes::Bytes;
use crate::{Connection, StreamEvent};

/// An event sent by the connection state machine.
pub enum ConnectionEvent {
    /// A message was received.
    ReceivedMessage(ChannelMessage),

    /// A stream event.
    StreamEvent(StreamEvent),

    /// Transmit a datagram.
    TransmitDatagram(Bytes),
}

/// An iterator of [`ConnectionEvent`] events from a [`Connection`].
pub struct ConnectionEventIter<'a> {
    inner: &'a mut Connection,
}

impl<'a> ConnectionEventIter<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        Self { inner: connection }
    }
}

impl Iterator for ConnectionEventIter<'_> {
    type Item = ConnectionEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.events.pop_front()
    }
}
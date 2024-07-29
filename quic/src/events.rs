use std::collections::VecDeque;
use bevy_stardust::prelude::ChannelMessage;
use bytes::Bytes;
use crate::StreamEvent;

/// An event sent by the connection state machine.
pub enum ConnectionEvent {
    /// A message was received.
    ReceivedMessage(ChannelMessage),

    /// A stream event.
    StreamEvent(StreamEvent),

    /// Transmit a datagram.
    TransmitDatagram(Bytes),
}

pub(crate) struct ConnectionEventQueue {
    events: VecDeque<ConnectionEvent>,
}

impl ConnectionEventQueue {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    pub fn push(&mut self, event: ConnectionEvent) {
        self.events.push_back(event)
    }

    fn pop(&mut self) -> Option<ConnectionEvent> {
        self.events.pop_front()
    }
}

/// An iterator of [`ConnectionEvent`] events from a [`Connection`].
pub struct ConnectionEventIter<'a> {
    events: &'a mut ConnectionEventQueue,
}

impl<'a> ConnectionEventIter<'a> {
    pub(crate) fn new(events: &'a mut ConnectionEventQueue) -> Self {
        Self { events }
    }
}

impl Iterator for ConnectionEventIter<'_> {
    type Item = ConnectionEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.events.pop()
    }
}
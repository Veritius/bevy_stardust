use std::collections::VecDeque;
use bevy_stardust::prelude::ChannelMessage;
use bytes::Bytes;
use crate::StreamEvent;

/// An event sent by the connection state machine.
pub enum ConnectionEvent {
    /// Returned when the remote connection behaved strangely,
    /// and that the connection must be closed.
    Overheated,

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
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(cap)
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
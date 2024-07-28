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
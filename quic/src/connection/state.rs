use std::collections::VecDeque;
use bevy_stardust::messages::ChannelMessage;
use super::{datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams}, events::StreamEvent, streams::{ChannelStreams, IncomingStreams, OutgoingStreams}};

pub struct ConnectionStateMachine {
    incoming_streams: IncomingStreams,
    outgoing_streams: OutgoingStreams,
    channel_streams: ChannelStreams,

    incoming_datagrams: IncomingDatagrams,
    outgoing_datagrams: OutgoingDatagrams,
    channel_datagrams: ChannelDatagrams,

    outgoing_messages: VecDeque<ChannelMessage>,
}

impl ConnectionStateMachine {
    pub fn recv_stream_event(&mut self, event: StreamEvent) {
        
    }
}
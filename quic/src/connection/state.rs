use std::collections::VecDeque;
use bevy_stardust::messages::ChannelMessage;
use super::{datagrams::{ChannelDatagrams, IncomingDatagrams, OutgoingDatagrams}, streams::{ChannelStreams, IncomingStreams, OutgoingStreams}};

pub struct ConnectionStateMachine {
    incoming_streams: IncomingStreams,
    outgoing_streams: OutgoingStreams,
    channel_streams: ChannelStreams,

    incoming_datagrams: IncomingDatagrams,
    outgoing_datagrams: OutgoingDatagrams,
    channel_datagrams: ChannelDatagrams,

    outgoing_messages: VecDeque<ChannelMessage>,
}
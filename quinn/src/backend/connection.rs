use bevy_stardust::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use quinn_proto::{ConnectionEvent, EndpointEvent};

pub(super) struct Connection {
    quinn_state: quinn_proto::Connection,

    conn_events: Receiver<ConnectionEvent>,
    endp_events: Sender<EndpointEvent>,

    inc_messages: Receiver<ChannelMessage>,
    out_messages: Sender<ChannelMessage>,
}

pub(crate) struct ConnectionHandle {
    inc_messages: Sender<ChannelMessage>,
    out_messages: Receiver<ChannelMessage>,
}
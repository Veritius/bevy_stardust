use std::collections::{BTreeMap, VecDeque};
use bevy_stardust::prelude::ChannelId;
use hashbrown::HashMap;
use crate::{ConnectionEvent, IncomingStream, RecvStreamId, SendStreamId};

/// The core state machine type, representing one QUIC connection.
pub struct Connection {
    pub(crate) events: VecDeque<ConnectionEvent>,

    pub(crate) incoming_streams: HashMap<RecvStreamId, IncomingStream>,

    pub(crate) outgoing_streams_channels: BTreeMap<ChannelId, SendStreamId>,
}

impl Connection {
    /// Creates a new [`Connection`] instance.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),

            incoming_streams: HashMap::new(),
            outgoing_streams_channels: BTreeMap::new(),
        }
    }

    /// Returns an event if one has occurred.
    pub fn poll(&mut self) -> Option<ConnectionEvent> {
        self.events.pop_front()
    }
}
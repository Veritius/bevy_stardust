use bevy::utils::HashMap;
use bevy_stardust::prelude::*;
use quinn_proto::StreamId;
use crate::{datagrams::*, streams::*};

pub(super) struct OutgoingShared {
    channels: HashMap<ChannelId, StreamId>,
}

impl OutgoingShared {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }
}

pub(super) struct OutgoingStreams {
    senders: HashMap<StreamId, Box<Send>>,
}

impl OutgoingStreams {
    pub fn new() -> Self {
        Self {
            senders: HashMap::new(),
        }
    }

    pub fn sender(&mut self, stream: StreamId) -> Option<OutgoingStream> {
        Some(OutgoingStream { send: self.senders.get_mut(&stream)? })
    }
}

pub(super) struct OutgoingStream<'a> {
    send: &'a mut Send,
}

impl<'a> StreamWriter for &mut OutgoingStream<'a> {
    #[inline]
    fn write<S: WritableStream>(self, stream: &mut S) -> StreamWriteOutcome {
        self.send.write(stream)
    }
}

impl<'a> OutgoingStream<'a> {
    #[inline]
    pub fn push(&mut self, chunk: Bytes) {
        self.send.push(chunk)
    }

    pub fn poll(&mut self) -> Option<OutgoingStreamEvent> {
        None
    }
}

pub(super) enum OutgoingStreamEvent {
    Finished,
}

pub(super) struct OutgoingDatagrams {
    sequencers: HashMap<ChannelId, DatagramSequencer>,
}

impl OutgoingDatagrams {
    pub fn new() -> Self {
        Self {
            sequencers: HashMap::new(),
        }
    }
}
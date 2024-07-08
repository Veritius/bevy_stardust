use bevy::utils::HashMap;
use bevy_stardust::prelude::*;
use quinn_proto::StreamId;
use crate::{datagrams::*, streams::*};

pub(super) struct OutgoingChannels {
    channels: HashMap<ChannelId, StreamId>,
}

pub(super) struct OutgoingStreams {
    senders: HashMap<StreamId, Box<Send>>,
}

pub(super) struct OutgoingDatagrams {
    sequencers: HashMap<ChannelId, DatagramSequencer>,
}
use bevy::utils::HashMap;
use bevy_stardust::messages::*;
use quinn_proto::StreamId;
use crate::{datagrams::*, streams::*};

pub(super) struct IncomingStreams {
    readers: HashMap<StreamId, Box<Recv>>,
}

pub(super) struct IncomingDatagrams {
    desequencers: HashMap<ChannelId, DatagramDesequencer>,
}

pub(super) struct HeldMessages {
    inner: Vec<ChannelMessage>,
}
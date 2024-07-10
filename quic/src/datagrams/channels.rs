use bevy::utils::HashMap;
use bevy_stardust::messages::ChannelId;
use bevy_stardust_extras::numbers::Sequence;

pub(crate) struct ChannelDatagrams {
    recv_sequences: HashMap<ChannelId, Sequence<u16>>,
    send_sequences: HashMap<ChannelId, Sequence<u16>>,
}

impl ChannelDatagrams {
    pub fn new() -> Self {
        Self {
            recv_sequences: HashMap::new(),
            send_sequences: HashMap::new(),
        }
    }
}
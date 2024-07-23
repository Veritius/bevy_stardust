use bevy::utils::HashMap;
use bevy_stardust::prelude::ChannelId;
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

    pub fn next_send_seq(&mut self, channel: ChannelId) -> Sequence<u16> {
        let value = self.send_sequences
            .entry(channel)
            .or_insert(Sequence::default());

        let ret = *value;
        value.increment();

        return ret;
    }
}
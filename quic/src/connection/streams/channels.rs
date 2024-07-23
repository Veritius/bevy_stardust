use bevy::utils::HashMap;
use bevy_stardust::prelude::ChannelId;
use super::StreamId;

pub(crate) struct ChannelStreams {
    cid_to_sid: HashMap<ChannelId, StreamId>,
    sid_to_cid: HashMap<StreamId, ChannelId>,
}

impl ChannelStreams {
    pub fn new() -> Self {
        Self {
            cid_to_sid: HashMap::new(),
            sid_to_cid: HashMap::new(),
        }
    }

    pub fn register(&mut self, cid: ChannelId, sid: StreamId) {
        self.cid_to_sid.insert(cid, sid);
        self.sid_to_cid.insert(sid, cid);
    }

    pub fn deregister_by_cid(&mut self, cid: ChannelId) {
        if let Some(sid) = self.cid_to_sid.remove(&cid) {
            self.sid_to_cid.remove(&sid);
        }
    }

    pub fn deregister_by_sid(&mut self, sid: StreamId) {
        if let Some(cid) = self.sid_to_cid.remove(&sid) {
            self.cid_to_sid.remove(&cid);
        }
    }
}
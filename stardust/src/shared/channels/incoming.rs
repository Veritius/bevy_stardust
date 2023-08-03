use std::collections::BTreeMap;
use bevy::prelude::*;
use crate::shared::{channels::id::ChannelId, payload::Payloads};

/// Incoming messages from this remote peer.
#[derive(Component)]
pub struct IncomingNetworkMessages(pub BTreeMap<ChannelId, Payloads>);

impl std::fmt::Debug for IncomingNetworkMessages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IncomingNetworkMessages(field hidden)")
    }
}
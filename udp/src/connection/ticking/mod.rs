use bevy_stardust::prelude::*;
use super::*;

impl ConnectionInner {
    /// Ticks the connection, parsing incoming data.
    pub(super) fn tick_preupdate(&mut self, context: PreUpdateTickData) {

    }

    /// Ticks the connection, queuing outgoing data.
    pub(super) fn tick_postupdate(&mut self, context: PostUpdateTickData) {

    }
}

/// Data used by [`tick_preupdate`](ConnectionInner::tick_preupdate)
pub(super) struct PreUpdateTickData<'a> {
    pub registry: &'a ChannelRegistryInner,
    pub messages: Option<Mut<'a, NetworkMessages<Incoming>>>,
}

/// Data used by [`tick_postupdate`](ConnectionInner::tick_postupdate)
pub(super) struct PostUpdateTickData<'a> {
    pub registry: &'a ChannelRegistryInner,
    pub messages: Option<Ref<'a, NetworkMessages<Outgoing>>>,
}
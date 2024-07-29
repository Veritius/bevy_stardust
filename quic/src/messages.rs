use bevy_stardust::{channels::ChannelRegistry, messages::MessageQueue, prelude::*};
use crate::Connection;

/// Context object required to handle outgoing messages.
pub struct RecvContext<'a> {
    /// A reference to the application's channel registry.
    pub registry: &'a ChannelRegistry,
}

impl Connection {
    /// Handle outgoing messages from a [`PeerMessages<Outgoing>`] component.
    #[inline]
    pub fn handle_outgoing<'a>(
        &'a mut self,
        context: RecvContext<'a>,
        queue: &'a PeerMessages<Outgoing>,
    ) {
        self.handle_outgoing_queue(
            context,
            queue.as_ref()
        )
    }

    /// Handles outgoing messages from a [`MessageQueue`].
    /// 
    /// If possible, you should use [`handle_outgoing`](Self::handle_outgoing) instead.
    pub fn handle_outgoing_queue<'a>(
        &'a mut self,
        context: RecvContext<'a>,
        queue: &'a MessageQueue,
    ) {
        todo!()
    }

    /// Handles outgoing [`ChannelMessage`] items from an iterator.
    pub fn handle_outgoing_iter<'a, I>(
        &'a mut self,
        context: RecvContext<'a>,
        iter: I,
    ) where
        I: IntoIterator<Item = ChannelMessage>,
    {

    }

    /// Handles outgoing messages on a specific channel from an iterator.
    pub fn handle_outgoing_channel_iter<'a, I>(
        &'a mut self,
        context: RecvContext<'a>,
        channel: ChannelId,
        iter: I,
    ) where
        I: IntoIterator<Item = Message>,
    {
        todo!()
    }
}
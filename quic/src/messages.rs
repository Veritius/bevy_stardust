use bevy_stardust::{channels::ChannelRegistry, messages::MessageQueue, prelude::*};
use crate::Connection;

/// Context object required to handle outgoing messages.
#[derive(Clone, Copy)]
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
        for (channel, messages) in queue.iter() {
            self.handle_outgoing_iter(context, channel, messages);
        }
    }

    /// Handles outgoing messages on a specific channel from an iterator.
    pub fn handle_outgoing_iter<'a, I>(
        &'a mut self,
        context: RecvContext<'a>,
        channel: ChannelId,
        iter: I,
    ) where
        I: IntoIterator<Item = Message>,
    {
        let config = match context.registry.config(channel) {
            Some(config) => config,
            None => todo!(),
        };

        for message in iter {
            self.handle_outgoing_inner(
                context,
                config,
                message,
            );
        }
    }

    fn handle_outgoing_inner<'a>(
        &'a mut self,
        context: RecvContext<'a>,
        config: &'a ChannelConfiguration,
        message: Message,
    ) {

    }
}
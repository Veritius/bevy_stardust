use bevy_stardust::{channels::ChannelRegistry, messages::MessageQueue, prelude::*};
use crate::Connection;

/// Context object required to handle outgoing messages.
#[derive(Clone, Copy)]
pub struct SendContext<'a> {
    /// A reference to the application's channel registry.
    pub registry: &'a ChannelRegistry,

    /// The maximum size of sent datagrams.
    pub dgram_max_size: usize,
}

impl Connection {
    /// Handle outgoing messages from a [`PeerMessages<Outgoing>`] component.
    #[inline]
    pub fn handle_outgoing<'a>(
        &'a mut self,
        context: SendContext<'a>,
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
        context: SendContext<'a>,
        queue: &'a MessageQueue,
    ) {
        for (channel, messages) in queue.iter() {
            self.handle_outgoing_iter(context, channel, messages);
        }
    }

    /// Handles outgoing messages on a specific channel from an iterator.
    pub fn handle_outgoing_iter<'a, I>(
        &'a mut self,
        context: SendContext<'a>,
        channel: ChannelId,
        iter: I,
    ) where
        I: IntoIterator<Item = Message>,
    {
        let config = match context.registry.config(channel) {
            Some(config) => config,
            None => todo!(),
        };

        match config.consistency {
            ChannelConsistency::UnreliableUnordered => self.handle_outgoing_inner(context, iter.into_iter(), Self::handle_outgoing_unrel_unord),
            ChannelConsistency::UnreliableSequenced => self.handle_outgoing_inner(context, iter.into_iter(), Self::handle_outgoing_unrel_seq),
            ChannelConsistency::ReliableUnordered => self.handle_outgoing_inner(context, iter.into_iter(), Self::handle_outgoing_rel_unord),
            ChannelConsistency::ReliableOrdered => self.handle_outgoing_inner(context, iter.into_iter(), Self::handle_outgoing_rel_ord),

            // We don't actually know what constraints new consistencies have, but reliable ordered is probably a good guess
            _ => self.handle_outgoing_inner(context, iter.into_iter(), Self::handle_outgoing_rel_ord),
        }
    }

    #[inline]
    fn handle_outgoing_inner<'a, I, F>(
        &'a mut self,
        context: SendContext<'a>,
        iter: I,
        func: F,
    ) where
        I: Iterator<Item = Message>,
        F: for<'f> Fn(&'f mut Self, SendContext<'f>, Message),
    {
        for message in iter {
            func(self, context, message)
        }
    }

    fn handle_outgoing_unrel_unord<'a>(
        &'a mut self,
        context: SendContext<'a>,
        message: Message,
    ) {
        todo!()
    }

    fn handle_outgoing_unrel_seq<'a>(
        &'a mut self,
        context: SendContext<'a>,
        message: Message,
    ) {
        todo!()
    }

    fn handle_outgoing_rel_unord<'a>(
        &'a mut self,
        context: SendContext<'a>,
        message: Message,
    ) {
        todo!()
    }

    fn handle_outgoing_rel_ord<'a>(
        &'a mut self,
        context: SendContext<'a>,
        message: Message,
    ) {
        todo!()
    }
}
use super::*;

/// Data used by [`tick_postupdate`](ConnectionStateMachine::tick_postupdate)
pub(in crate::connection) struct PostUpdateTickData<'a> {
    pub config: &'a PluginConfiguration,
    pub registry: &'a ChannelRegistryInner,
    pub messages: Option<Ref<'a, NetworkMessages<Outgoing>>>,
}

impl ConnectionStateMachine {
    pub fn tick_postupdate(
        &mut self,
        shared: &mut ConnectionShared,
        context: PostUpdateTickData,
    ) {
        match &mut self.inner {
            MachineInner::Handshaking(machine) => {
                // Repeatedly call send until it returns None (nothing to send)
                // This acts very similarly to an iterator
                while let Some(bytes) = machine.send(context.config, shared) {
                    shared.send_queue.push_back(bytes);
                }
            },

            MachineInner::Established(machine) => {
                machine.tick_postupdate(shared, context);
            },

            MachineInner::Closed => {}
        }
    }
}
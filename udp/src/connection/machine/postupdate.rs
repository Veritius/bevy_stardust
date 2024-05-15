use super::*;

/// Data used by [`tick_postupdate`](ConnectionInner::tick_postupdate)
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
        'outer: loop {
            match &mut self.inner {
                MachineInner::Handshaking(_) => todo!(),
                MachineInner::Established => todo!(),
                MachineInner::Closing => todo!(),
                MachineInner::Closed => { break 'outer }
            }
        }
    }
}
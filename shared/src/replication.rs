use crate::channel::Channel;

/// Channel for component replication messages.
#[derive(Debug)]
pub(crate) struct ComponentReplicationChannel;
impl Channel for ComponentReplicationChannel {}
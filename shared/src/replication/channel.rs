use crate::channel::Channel;

/// Channel for component replication messages.
#[derive(Debug)]
pub struct ComponentReplicationChannel;
impl Channel for ComponentReplicationChannel {}
use std::marker::PhantomData;
use bevy::prelude::{Resource, Component};
use crate::channel::ChannelConfig;

/// Configuration for `ReplicateComponentPlugin` plugins.
#[derive(Debug)]
pub struct ComponentReplicationConfig {
    /// Whether or not the component is replicated by default.
    /// 
    /// This can be further controlled per-entity using the `AllowReplication<T>` and `PreventReplication<T>` marker components.
    pub replication_mode: ComponentReplicationMode,
    pub channel_config_override: Option<ChannelConfig>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentReplicationMode {
    /// This component is replicated by default, but will not be replicated if `PreventReplication<T>` is present.
    ReplicatedByDefault,
    /// This component is not replicated by default, but will be replicated if `AllowReplication<T>` is present.
    UnreplicatedByDefault,
    /// This component will **ignore** the `AllowReplication<T>` and `PreventReplication<T>` components, and will always be replicated.
    AlwaysReplicate,
}

#[derive(Resource)]
pub(crate) struct ComponentReplicationIdDispenser(pub u16);
impl ComponentReplicationIdDispenser {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn new_id(&mut self) -> u16 {
        if self.0 == u16::MAX { panic!("Component replication ID limit exceeded!") }
        let id = self.0;
        self.0 += 1;
        return id
    }
}

#[derive(Resource)]
pub(crate) struct ReplicatedComponentData<T: Component> {
    pub replication_id: u16,
    pub replicate_by_default: bool,
    pub phantom: PhantomData<T>,
}
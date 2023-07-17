use std::marker::PhantomData;
use bevy::prelude::{Resource, Component};
use crate::channel::ChannelConfig;

/// Configuration for `ReplicateComponentPlugin` plugins.
#[derive(Debug, Clone)]
pub struct ComponentReplicationConfig {
    /// Whether or not the component is replicated by default.
    /// 
    /// This can be further controlled per-entity using the `AllowReplication<T>` and `PreventReplication<T>` marker components.
    pub replication_mode: ComponentReplicationMode,
    pub channel_config_override: Option<ChannelConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComponentReplicationMode {
    /// This component is replicated by default, but will not be replicated if `PreventReplication<T>` is present.
    ReplicatedByDefault,
    /// This component is not replicated by default, but will be replicated if `AllowReplication<T>` is present.
    UnreplicatedByDefault,
    /// This component will **ignore** the `AllowReplication<T>` and `PreventReplication<T>` components, and will always be replicated.
    AlwaysReplicate,
}

#[derive(Resource)]
pub(crate) struct ReplicatedComponentData<T: Component> {
    pub config: ComponentReplicationConfig,
    pub phantom: PhantomData<T>,
}
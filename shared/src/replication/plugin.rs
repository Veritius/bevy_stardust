use std::marker::PhantomData;
use bevy::prelude::*;
use crate::bits::ManualBitSerialisation;
use super::config::ComponentReplicationConfig;

/// Enables replication for a component implementing `ManualBitSerialisation`.
/// 
/// If your component does not implement `ManualBitSerialisation`, use `ReplicateComponentPluginReflected`.
/// You should only need to do this for components from other crates. If possible, you should use this version.
pub struct ReplicateComponentPluginBitstream<T: Component + ManualBitSerialisation> {
    config: ComponentReplicationConfig,
    phantom: PhantomData<T>
}

impl<T: Component + ManualBitSerialisation> ReplicateComponentPluginBitstream<T> {
    pub fn new(config: ComponentReplicationConfig) -> Self {
        Self { config, phantom: PhantomData }
    }
}

impl<T: Component + ManualBitSerialisation> Plugin for ReplicateComponentPluginBitstream<T> {
    fn build(&self, app: &mut App) {
        todo!()
    }

    fn finish(&self, app: &mut App) {
        todo!()
    }
}

/// Enables replication for a component implementing `Reflect`.
/// 
/// If possible, you should use `ReplicateComponentPluginBitstream` as `ManualBitSerialisation` implementors are sent more efficiently.
pub struct ReplicateComponentPluginReflected<T: Component + Reflect> {
    config: ComponentReplicationConfig,
    phantom: PhantomData<T>
}

impl<T: Component + Reflect> ReplicateComponentPluginReflected<T> {
    pub fn new(config: ComponentReplicationConfig) -> Self {
        Self { config, phantom: PhantomData }
    }
}

impl<T: Component + Reflect> Plugin for ReplicateComponentPluginReflected<T> {
    fn build(&self, app: &mut App) {
        todo!()
    }

    fn finish(&self, app: &mut App) {
        todo!()
    }
}

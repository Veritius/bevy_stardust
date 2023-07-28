use std::marker::PhantomData;
use bevy::prelude::*;
use crate::shared::{serialisation::ManualBitSerialisation, channels::{extension::ChannelSetupAppExt, components::{ChannelConfig, ChannelDirection}}};
use super::{config::{ComponentReplicationConfig, ReplicatedComponentData}, systems::{replication_send_system_reflected, replication_send_system_bitstream}, channel::ComponentReplicationChannel};

/// Enables replication for a component implementing `ManualBitSerialisation`.
/// 
/// If your component does not implement `ManualBitSerialisation`, use `ReplicateComponentPluginReflected`.
/// You should only need to do this for components from other crates. If possible, you should use this version.
pub struct ReplicateComponentPluginBitstream<T: Component + ManualBitSerialisation> {
    config: ComponentReplicationConfig,
    phantom: PhantomData<T>
}

impl<T: Component + ManualBitSerialisation + std::fmt::Debug> ReplicateComponentPluginBitstream<T> {
    pub fn new(config: ComponentReplicationConfig) -> Self {
        Self { config, phantom: PhantomData }
    }
}

impl<T: Component + ManualBitSerialisation + std::fmt::Debug> Plugin for ReplicateComponentPluginBitstream<T> {
    fn build(&self, app: &mut App) {
        app.register_channel::<T>(ChannelConfig { direction: ChannelDirection::ServerToClient }, ());

        app.insert_resource(ReplicatedComponentData {
            config: self.config.clone(),
            phantom: PhantomData::<T>,
        });
        
        app.add_systems(PostUpdate, (
            replication_send_system_bitstream::<T>,
        ));
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

impl<T: Component + Reflect + std::fmt::Debug> ReplicateComponentPluginReflected<T> {
    pub fn new(config: ComponentReplicationConfig) -> Self {
        Self { config, phantom: PhantomData }
    }
}

impl<T: Component + Reflect + std::fmt::Debug> Plugin for ReplicateComponentPluginReflected<T> {
    fn build(&self, app: &mut App) {
        app.register_channel::<T>(ChannelConfig { direction: ChannelDirection::ServerToClient }, ());

        app.insert_resource(ReplicatedComponentData {
            config: self.config.clone(),
            phantom: PhantomData::<T>,
        });

        app.add_systems(PostUpdate, (
            replication_send_system_reflected::<T>,
        ));
    }

    fn finish(&self, app: &mut App) {
        todo!()
    }
}
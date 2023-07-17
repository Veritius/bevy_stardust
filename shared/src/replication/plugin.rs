use std::marker::PhantomData;
use bevy::prelude::*;
use crate::{bits::ManualBitSerialisation, schedule::NetworkTransmit, protocol::ProtocolAppExts, channel::{ChannelConfig, ChannelDirection, ChannelOrdering, ChannelReliability, ChannelErrorChecking, ChannelFragmentation, ChannelCompression, ChannelEncryption, ChannelLatestness}};
use super::{config::{ComponentReplicationConfig, ReplicatedComponentData}, systems::{replication_send_system_reflected, replication_send_system_bitstream}, channel::ComponentReplicationChannel};

const DEFAULT_REPLICATION_CHANNEL_CONFIG: ChannelConfig = ChannelConfig {
    direction: ChannelDirection::Bidirectional,
    ordering: ChannelOrdering::Unordered,
    reliability: ChannelReliability::Reliable,
    latestness: ChannelLatestness::Within(10),
    error_checking: ChannelErrorChecking::Enabled,
    fragmentation: ChannelFragmentation::Enabled,
    compression: ChannelCompression::Disabled,
    encryption: ChannelEncryption::Signing,
};

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
        app.add_net_channel::<ComponentReplicationChannel<T>>(
            if let Some(config) = &self.config.channel_config_override {
                // return override value
                config.clone()
            } else {
                // default config for replication channels
                DEFAULT_REPLICATION_CHANNEL_CONFIG
            }
        );

        app.insert_resource(ReplicatedComponentData {
            config: self.config.clone(),
            phantom: PhantomData::<T>,
        });
        
        app.add_systems(NetworkTransmit::Process, (
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

impl<T: Component + Reflect> ReplicateComponentPluginReflected<T> {
    pub fn new(config: ComponentReplicationConfig) -> Self {
        Self { config, phantom: PhantomData }
    }
}

impl<T: Component + Reflect> Plugin for ReplicateComponentPluginReflected<T> {
    fn build(&self, app: &mut App) {
        app.add_net_channel::<ComponentReplicationChannel<T>>(
            if let Some(config) = &self.config.channel_config_override {
                // return override value
                config.clone()
            } else {
                // default config for replication channels
                DEFAULT_REPLICATION_CHANNEL_CONFIG
            }
        );

        app.insert_resource(ReplicatedComponentData {
            config: self.config.clone(),
            phantom: PhantomData::<T>,
        });

        app.add_systems(NetworkTransmit::Process, (
            replication_send_system_reflected::<T>,
        ));
    }

    fn finish(&self, app: &mut App) {
        todo!()
    }
}
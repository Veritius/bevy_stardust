use bevy::{prelude::*, reflect::Reflect};
use crate::{bits::ManualBitSerialisation, protocol::ProtocolAppExts, channel::{ChannelConfig, ChannelDirection, ChannelOrdering, ChannelReliability, ChannelErrorChecking, ChannelFragmentation, ChannelCompression, ChannelEncryption, ChannelSigning}, replication::{config::ReplicatedComponentData, systems::{reflection_send_system_bitstream, replication_send_system_reflected}}, schedule::NetworkTransmit};
use super::{channel::ComponentReplicationChannel, config::ComponentReplicationIdDispenser};

/// Marker resource for ensuring the component replication channel isn't added twice.
#[derive(Resource)]
pub(crate) struct AutoReplicationEnabled;

pub trait ReplicationAppExts {
    /// Enable automatic replication for components that implement `ManualBitSerialisation`
    fn replicate_bitstream_component<T: Component + ManualBitSerialisation>(&mut self, replicate_by_default: bool);
    /// Enable automatic replication for components that implement `Reflect`
    /// This is worse than `replicate_bitstream_component`, and should only be used for types that don't implement `ManualBitSerialisation`
    fn replicate_reflected_component<T: Component + Reflect>(&mut self, replicate_by_default: bool);
}

fn replication_channel(app: &mut App) {
    if !app.world.contains_resource::<AutoReplicationEnabled>() {
        app.add_net_channel::<ComponentReplicationChannel>(ChannelConfig {
            direction: ChannelDirection::Bidirectional,
            ordering: ChannelOrdering::Ordered,
            reliability: ChannelReliability::Reliable,
            error_checking: ChannelErrorChecking::Enabled,
            fragmentation: ChannelFragmentation::Enabled,
            compression: ChannelCompression::Disabled,
            encryption: ChannelEncryption::Disabled,
            signing: ChannelSigning::Enabled,
            latest_only: true,
        });

        app.insert_resource(ComponentReplicationIdDispenser::new());
    }
}

fn add_replication_data<T: Component>(app: &mut App, default: bool) {
    let id = app.world.resource_mut::<ComponentReplicationIdDispenser>().new_id();
    app.insert_resource(ReplicatedComponentData::<T> {
        replication_id: id,
        replicate_by_default: default,
        phantom: std::marker::PhantomData,
    });
}

impl ReplicationAppExts for App {
    fn replicate_bitstream_component<T: Component + ManualBitSerialisation>(&mut self, replicate_by_default: bool) {
        replication_channel(self);
        add_replication_data::<T>(self, replicate_by_default);
        self.add_systems(NetworkTransmit::Transmit, (
            reflection_send_system_bitstream::<T>,
        ));
    }

    fn replicate_reflected_component<T: Component + Reflect>(&mut self, replicate_by_default: bool) {
        replication_channel(self);
        add_replication_data::<T>(self, replicate_by_default);
        self.add_systems(NetworkTransmit::Transmit, (
            replication_send_system_reflected::<T>,
        ));
    }
}
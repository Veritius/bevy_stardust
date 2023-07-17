use bevy::prelude::{Plugin, App};
use crate::{protocol::{ProtocolBuilder, ProtocolAppExts}, channel::{ChannelConfig, ChannelDirection, ChannelOrdering, ChannelReliability, ChannelLatestness, ChannelErrorChecking, ChannelFragmentation, ChannelCompression, ChannelEncryption, ChannelSigning}, authentication::AuthenticationChannel};

pub struct StardustSharedPlugin;
impl Plugin for StardustSharedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProtocolBuilder::default());
        app.add_net_channel::<AuthenticationChannel>(ChannelConfig {
            direction: ChannelDirection::Bidirectional,
            ordering: ChannelOrdering::Ordered,
            reliability: ChannelReliability::Reliable,
            latestness: ChannelLatestness::Ignore,
            error_checking: ChannelErrorChecking::Enabled,
            fragmentation: ChannelFragmentation::Enabled,
            compression: ChannelCompression::Disabled,
            encryption: ChannelEncryption::Disabled,
            signing: ChannelSigning::Enabled,
        });
    }

    fn finish(&self, app: &mut App) {
        let builder = app.world.remove_resource::<ProtocolBuilder>()
            .expect("Builder should have been present").build();
        app.world.insert_resource(builder);
    }
}
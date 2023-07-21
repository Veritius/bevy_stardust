use bevy::prelude::{Plugin, App, warn, info};
use crate::{protocol::{ProtocolBuilder, ProtocolAppExts}, channel::{ChannelConfig, ChannelDirection, ChannelOrdering, ChannelReliability, ChannelLatestness, ChannelErrorChecking, ChannelFragmentation, ChannelCompression, ChannelEncryption}};

/// Shared information between the client and server.
/// See the demos for information on how to use this.
pub struct StardustSharedPlugin {}
impl Plugin for StardustSharedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProtocolBuilder::default());
    }

    fn finish(&self, app: &mut App) {
        let protocol = app.world.remove_resource::<ProtocolBuilder>()
            .expect("Builder should have been present").build();

        // if protocol.any_encrypted() && !app.world.contains_resource::<AuthServerLocation>() {
        //     warn!("One or more channels in the protocol have cryptographic features enabled, but no shared authority is defined. Encryption and signing are disabled!");
        // }

        info!("Protocol ID set to {}", protocol.id());
        app.world.insert_resource(protocol);
    }
}
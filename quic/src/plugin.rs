use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::connections::*;
use crate::endpoints::*;

/// Adds QUIC functionality to the application.
pub struct QuicPlugin {
    /// The maximum length of framed messages, applying to Stardust messages.
    /// 
    /// Due to how framed messages are handled within QUIC streams,
    /// denial-of-service attacks are possible. This variable allows
    /// this attack vector to be mitigated. If this value is too low,
    /// your messages will be incorrectly identified as attacks.
    /// If it's too high, denial of service attacks become viable.
    /// 
    /// This should be as set as high as your app will ever use,
    /// and then a little bit more for wiggle room.
    /// If you need to send data that is extremely large,
    /// but you don't want to have this attack vector available,
    /// look into rolling your own fragmentation protocol.
    /// 
    /// Defaults to `2^16` (`65535`) bytes.
    pub maximum_framed_message_length: usize,
}

impl Default for QuicPlugin {
    fn default() -> Self {
        Self {
            maximum_framed_message_length: 2usize.pow(16),
        }
    }
}

impl Plugin for QuicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PluginConfig {
            max_frm_msg_len: self.maximum_framed_message_length,
        });

        app.add_systems(PreUpdate, (
            endpoint_datagram_recv_system,
            connection_endpoint_events_system,
            connection_event_handler_system,
        ).chain().in_set(NetworkRead::Receive));

        app.add_systems(PostUpdate, (
            connection_message_sender_system,
            connection_datagram_send_system,
            connection_endpoint_events_system,
        ).chain().in_set(NetworkWrite::Send));
    }
}

#[derive(Resource)]
pub(crate) struct PluginConfig {
    pub max_frm_msg_len: usize,
}
use std::time::Instant;

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::plugin::PluginConfiguration;
use crate::prelude::*;
use super::Established;

pub(crate) fn established_packet_reader_system(
    registry: Res<ChannelRegistry>,
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    todo!()
}

pub(crate) fn established_packet_builder_system(
    registry: Res<ChannelRegistry>,
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &NetworkMessages<Outgoing>)>,
) {
    todo!()
}

pub(crate) fn established_timeout_system(
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, Option<&mut NetworkPeerLifestage>)>,
) {
    connections.par_iter_mut().for_each(|(entity, mut connection, mut established, lifestage)| {
        let now = Instant::now();
        let last_recv = if let Some(last_recv) = connection.timings.last_recv { last_recv } else { connection.timings.started };

        // Disconnect them if they've timed out
        let timeout_dur = now.duration_since(last_recv);
        if timeout_dur > config.connection_timeout {
            // Update state information
            connection.state = ConnectionState::Closed;
            if let Some(mut lifestage) = lifestage {
                *lifestage = NetworkPeerLifestage::Closed;
            }

            // Log the disconnection
            tracing::debug!("Connection {entity:?} timed out after {} seconds", timeout_dur.as_secs());

            // Early return to prevent keep-alive check
            return;
        }

        // Send a keep-alive packet
        let last_sent = connection.timings.last_sent;
        if last_sent.is_some() && now.duration_since(last_sent.unwrap()) > config.keep_alive_timeout {
            todo!()
        }
    });
}
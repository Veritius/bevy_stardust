use std::time::Instant;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::plugin::PluginConfiguration;
use crate::prelude::*;
use super::Established;

/// Runs [`poll`](Established::poll) on all [`Established`] entities.
pub(crate) fn established_polling_system(
    registry: Res<ChannelRegistry>,
    config: Res<PluginConfiguration>,
    mut connections: Query<(&mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    connections.par_iter_mut().for_each(|(
        mut connection,
        mut established,
        messages
    )| {
        established.poll(
            &mut connection,
            messages,
            &registry,
            &config
        );
    })
}

pub(crate) fn established_timeout_system(
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &mut NetworkPeerLifestage)>,
) {
    connections.par_iter_mut().for_each(|(entity, mut connection, mut established, mut lifestage)| {
        let now = Instant::now();
        let last_recv = if let Some(last_recv) = connection.timings.last_recv { last_recv } else { connection.timings.started };

        // Disconnect them if they've timed out
        let timeout_dur = now.duration_since(last_recv);
        if timeout_dur > config.connection_timeout {
            // Update state information
            connection.is_closing = false;
            connection.close_reason = None;
            connection.fully_closed = true;
            *lifestage = NetworkPeerLifestage::Closed;

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

pub(crate) fn established_closing_system(
    mut events: EventReader<DisconnectPeerEvent>,
    mut peers: Query<(&mut Connection, &mut Established)>,
) {
    for event in events.read() {
        // Access the target (might fail, so we handle that)
        let (mut connection, mut established) = match peers.get_mut(event.peer) {
            Ok(v) => v,
            Err(_) => { continue; },
        };

        todo!()
    }
}
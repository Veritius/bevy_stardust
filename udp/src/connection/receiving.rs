use bevy_ecs::prelude::*;
use crate::Connection;

use super::statemachine::ConnectionStateMachine;

pub(crate) fn connection_packet_processing_system(
    mut connections: Query<&mut Connection>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|mut connection| {
        // Check if they've actually sent anything before continuing
        if connection.packet_queue.incoming().is_empty() { return }

        /*
            If encryption or other processing was to be added, decryption would go here.
            At this point, the packets are exactly as received over the wire.
        */

        // Process bytes based on the state machine
        match connection.state_machine {
            ConnectionStateMachine::Handshaking(_) => todo!(),
            ConnectionStateMachine::Established => todo!(),
            ConnectionStateMachine::Closing => todo!(),
            ConnectionStateMachine::Closed => todo!(),
        }
    });
}
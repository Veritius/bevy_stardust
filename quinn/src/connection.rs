use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::Connection as ConnectionState;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use quinn::Connecting;

/// Represents one Quinn connection.
#[derive(Component)]
pub struct QuinnConnection {
    handle: ConnectionHandle,
    sp_state: Box<ConnectionState>,
}

impl QuinnConnection {
    pub(crate) fn connecting(connecting: Connecting) -> Self {
        Self {
            handle: todo!(),
            sp_state: Box::new(ConnectionState::new()),
        }
    }
}

struct ConnectionHandle {
    incoming_messages: Receiver<ChannelMessage>,
    outgoing_messages: Sender<ChannelMessage>,
}

pub(crate) fn message_recv_system(
    mut connections: Query<(&mut QuinnConnection, &mut PeerMessages<Incoming>)>,
) {
    connections.par_iter_mut().for_each(|(mut connection, mut incoming)| {
        loop {
            match connection.handle.incoming_messages.try_recv() {
                Ok(message) => incoming.push_one(message),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => todo!(),
            }
        }
    });
}

pub(crate) fn message_send_system(
    mut connections: Query<(&mut QuinnConnection, &PeerMessages<Outgoing>)>,
) {
    connections.par_iter_mut().for_each(|(mut connection, outgoing)| {
        'outer: for (channel, messages) in outgoing.iter() {
            for message in messages {
                // Try to send the message, continue if successful (guard)
                let message = ChannelMessage { channel, message };
                if let Ok(_) = connection.handle.outgoing_messages.send(message) { continue }

                // An error occurred
                // TODO: Handle this case better
                break 'outer;
            }
        }
    });
}
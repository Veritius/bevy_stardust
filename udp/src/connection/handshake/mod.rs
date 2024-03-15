/*

This is an explanation of how the standard handshake works.
For the purposes of the explanation, we will use the following terms.
- "Initiator" to refer to the peer making the outgoing connection, and acting like a client.
- "Listener" to refer to the peer receiving the incoming connection, and acting like a server.

While the overall handshake is not set in stone, a portion this initial reliability handshake is.
For the first packet, the first three fields will always remain the same.
For the second packet, the first four fields will always remain the same.
This is to ensure good errors when trying to connect to an old version of a game.

The first packet is sent by the Initiator to the Listener, to start a connection.
It consists of the following information, with square brackets showing the type.

[u64] Transport identifier
[u32] Transport major version
[u32] Transport minor version
[u64] Application identifier
[u32] Application major version
[u32] Application minor version
[u16] Sequence identifier
(total length 34 bytes)

The second packet is sent by the Listener to the Initiator, to acknowledge the connection response.
This establishes the acknowledgement state of the Initiator, and begins the same process for the Listener.

[u16] Response code
[u64] Transport identifier
[u32] Transport major version
[u32] Transport minor version
[u64] Application identifier
[u32] Application major version
[u32] Application minor version
[u16] Sequence identifier
[u16] Acknowledgement
[u16] Ack bitfield
(total length 40 bytes)

The third packet is sent by the Initiator to the Listener, to acknowledge the Initiator's sequence id.
This establishes the acknowledgement state of the Listener. At this point, both peers start communicating reliably.
[u16] Response code
[u16] Sequence identifier
[u16] Acknowledgement
[u16] Ack bitfield
(total length 8 bytes)

*/

mod codes;
mod impls;
mod packets;
mod system;

pub(crate) use system::{handshake_polling_system, potential_new_peers_system};

use std::{net::SocketAddr, time::Instant};
use bevy_ecs::prelude::*;
use crate::{Connection, ConnectionDirection};
use super::reliability::ReliabilityData;
use codes::HandshakeErrorCode;

#[derive(Bundle)]
pub(crate) struct OutgoingHandshake {
    pub connection: Connection,
    handshake: Handshaking,
}

impl OutgoingHandshake {
    pub fn new(
        owning_endpoint: Entity,
        remote_address: SocketAddr,
    ) -> Self {
        Self {
            connection: Connection::new(
                owning_endpoint,
                remote_address, 
                ConnectionDirection::Outgoing,
            ),
            handshake: Handshaking {
                started: Instant::now(),
                state: HandshakeState::ClientHello,
                reliability: ReliabilityData::new(),
            },
        }
    }
}

#[derive(Component)]
pub(crate) struct Handshaking {
    started: Instant,
    state: HandshakeState,
    reliability: ReliabilityData,
}

#[derive(Debug)]
enum HandshakeState {
    ClientHello,
    ServerHello,
    Finished,
    Failed(HandshakeFailureReason),
}

impl HandshakeState {
    pub fn is_end(&self) -> bool {
        use HandshakeState::*;
        match self {
            Finished | Failed(_) => true,
            _ => false,
        }
    }
}

impl From<HandshakeFailureReason> for HandshakeState {
    fn from(value: HandshakeFailureReason) -> Self {
        Self::Failed(value)
    }
}

#[derive(Debug)]
enum HandshakeFailureReason {
    TimedOut,
    BadResponse,
    WeRejected(HandshakeErrorCode),
    TheyRejected(HandshakeErrorCode),
}

impl std::fmt::Display for HandshakeFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use HandshakeFailureReason::*;
        match self {
            TimedOut => f.write_str("timed out"),
            BadResponse => f.write_str("remote peer invalid response packet"),
            WeRejected(error_code) => f.write_fmt(format_args!("rejected by remote peer: {error_code}")),
            TheyRejected(error_code) => f.write_fmt(format_args!("we rejected remote peer: {error_code}")),
        }
    }
}
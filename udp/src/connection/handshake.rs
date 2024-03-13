//! Handshake implementation. Based on TCP and QUIC.
//! This protocol is largely distinct from the rest of the code.

/*

This is an explanation of how the standard handshake works.
For the purposes of the explanation, we will use the following terms.
- "Initiator" to refer to the peer making the outgoing connection, and acting like a client.
- "Listener" to refer to the peer receiving the incoming connection, and acting like a server.

The first packet is sent by the Initiator to the Listener, to start a connection.
It consists of the following information, with square brackets showing the type.
[u64] Transport identifier
[u32] Transport version (minor)
[u32] Transport version (major)
[u64] Application identifier
[u16] Sequence identifier

The second packet is sent by the Listener to the Initiator, to acknowledge the connection response.
This establishes the acknowledgement state of the Initiator, and begins the same process for the Listener.
[u64] Transport identifier
[u32] Transport version (minor)
[u32] Transport version (major)
[u64] Application identifier
[u16] Response code
[u16] Sequence identifier
[u16] Acknowledgement
[uVr] Ack bitfield

The third packet is sent by the Initiator to the Listener, to acknowledge the Initiator's sequence id.
This establishes the acknowledgement state of the Listener. At this point, both peers start communicating reliably.
[u16] Response code
[u16] Sequence identifier
[u16] Acknowledgement
[uVr] Ack bitfield

*/

use crate::reliability::ReliabilityData;

/// Handshake state machine for connections.
#[derive(Debug)]
pub(super) struct ConnectionHandshake {
    context: HandshakeContext,
    machine: HandshakeState,
    reliability: ReliabilityData,
}

#[derive(Debug)]
pub(super) struct HandshakeContext {
    pub transport_identifier: u64,
    pub transport_version_minor: u32,
    pub transport_version_major: u32,
    pub application_identifier: u64,
}

#[derive(Debug)]
enum HandshakeState {
    RelSynSent,
    RelSynRecv,
    Finished,
    Failure,
}
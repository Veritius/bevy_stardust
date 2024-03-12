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
[u32] Transport version (breaking)
[u32] Transport version (non-breaking)
[u16] Sequence identifier

The second packet is sent by the Listener to the Initiator, to acknowledge the connection response.
This establishes the acknowledgement state of the Initiator, and begins the same process for the Listener.
[u64] Transport identifier
[u32] Transport version (breaking)
[u32] Transport version (non-breaking)
[u16] Response code
[u16] Sequence identifier
[u16] Acknowledgement
[uVr] Ack bitfield

The third packet is sent by the Initiator to the Listener, to acknowledge the Initiator's sequence id.
This establishes the acknowledgement state of the Listener.
[u16] Sequence identifier
[u16] Acknowledgement
[uVr] Ack bitfield

*/

/// Handshake state machine for connections.
pub(crate) struct ConnectionHandshake(());
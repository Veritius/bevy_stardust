# Stardust Protocol
**THIS IS NOT DONE**

## Messaging system
This is the protocol used to send messages between the client and server.

The implementation is inspired by the following resources:
- [Reliability and Congestion Avoidance over UDP - Gaffer on Games](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/)

If authentication is enabled, the server and client will run through the authentication process, documented below.

### Packet layouts
All packets start with a unique 32-bit 'protocol id'. This protocol ID is used to discard packets from anything that isn't your game.

Then, there is a 32-bit number indicating the **channel**. This is used to direct the right messages to the right Bevy systems. Channels, based on the protocol information, may have certain features enabled, like fragmentation or cryptography. In this case, extra information will be added that will be processed by Stardust, and the actual payload will be given to your Bevy systems.

6 bytes are dedicated to **acks**, documented in [this page by Gaffer on Games](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/). Despite being for reliability, they will be included in non-reliable channels.

In total, Stardust will use at least **14** bytes for a single packet.

#### Maybe-included data
Some information is not included in all packets.

**Sequence value**
If the channel is ordered or has cryptographic features, a 32-bit sequence value will be added. This is used to ensure messages arrive in a specific order and in the case of cryptographic features being enabled, prevents replay attacks.

**TODO: Channel reliability, latestness, error checking, fragmentation, encryption/signing**

### Channels
TODO

## Authentication system
This system exists to allow a Diffie-Hellman key exchange ~~without any chance of a MITM attack~~ (soon).
This only applies to the cryptographic authentication step. For client-server communication, see `Messaging system`.

The implementation is inspired by the following resources:
- [TLS Handshake Explained - Computerphile](https://www.youtube.com/watch?v=86cQJ0MMses)
- [netcode.io 1.02 protocol](https://github.com/networkprotocol/netcode/blob/997c0e67b84bf385e9789fd7d99942cbab216c6f/STANDARD.md)

For the purposes of this explanation:
- C is the client trying to join
- S is the dedicated server
- A is the authentication server, whose public key C and S already know

When a server first starts up:
1. S sends a message to A, containing S's public key. This message is encrypted using A's public key and contains a digest of the payload.
2. A signs the message verifying S's public key, sending it back to S.

When a client tries to join:
1. This process is the same as what the server does when it first starts up, but it's C instead of S.
2. C requests to join S, attaching its A-signed public key.
3. S checks that C's message is valid using A's public key, and sends back S's A-signed public key.
4. C checks that S's message is valid using A's public key.

### Packet layout
- "STARDUST AUTH 0.1.0" - auth version string, 20 bytes (may change), null-terminated
- 8 bit number - message type
- Payload digest (256 bits)
- Payload (up to 1500 bytes - everything before)

#### Message type IDs
- 001 - Request for public key signature
- 002 - Signed public key response
- 003 - Digest was invalid, resend
- 004 - Failed for indeterminate reason
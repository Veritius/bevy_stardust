# Stardust Protocol
**THIS IS NOT DONE**

## Messaging system
This is the protocol used to send messages between the client and server.

The implementation is inspired by the following resources:
- [Reliability and Congestion Avoidance over UDP - Gaffer on Games](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/)

### Packet layouts
All packets start with a unique 32-bit 'protocol id'. This protocol ID is used to discard packets from anything that isn't your game.

Then, there is a 24-bit number indicating the **channel**. This is used to direct the right messages to the right Bevy systems. In this case, extra information will be added that will be processed by Stardust, and the actual payload will be given to your Bevy systems.

6 bytes are dedicated to **acks**, documented in [this page by Gaffer on Games](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/). Despite being for reliability, they will be included in non-reliable channels.

In total, Stardust will use at least **14** bytes for a single packet.

#### Maybe-included data
Some information is not included in all packets.

**Sequence value**
If the channel is ordered, a 32-bit sequence value will be added. This is used to ensure messages arrive in a specific order.

**TODO: Channel reliability, latestness, error checking, fragmentation**

### Channels
TODO
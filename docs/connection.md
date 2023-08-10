# Stardust connection process
This process runs when a client tries to connect to a server.
This only applies to the default Stardust transport layers, ie UDP.

Transport layers from other crates may have their own process.

## UDP
### Packets
Packets in the UDP transport layer are broken into a header and payload section.

Each packet's header size is variable, but consistent based on a channel ID, which is always present. This allows information like sequence values to only be sent if actually needed.

#### All possible packet data
**Channel ID (3 bytes)**<br>
Used to indicate other header data in the packet, and to correctly compartmentalise the channels.

**Ack & ack bitfield (8 bytes)**<br>
Always present to ensure the functionality of reliablity for the entire system, even if the packet is sent over an unreliable channel.

See [Gaffer on Games' article about reliability over UDP](https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/) for more.

**Sequence id (optional, 2 bytes)**<br>
Used for ordering information, if the channel is ordered.

**Fragment id (optional, 2 bytes)**<br>
Used for message fragmentation.

**Payload (1500 - everything else bytes)**<br>
TODO
# Packets look like this (WIP)

See the following: https://gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/

Always present:
- 4 bytes: Protocol id (32-bit number)
- 2 bytes: Channel id (16-bit number)
- 2 bytes: Ack (16-bit number)
- 4 bytes: Ack bitfield (32-bit bitfield)

If message is being sent to the server:
- 2 bytes: Client id (16-bit number, client-to-server messages only)

If channel is ordered:
- 4 bytes: Sequence value (32-bit number)

The rest of the packet is payload.
Packets look like this (WIP)

- 4 bytes: Protocol id (32-bit number)
- 2 bytes: Channel id (16-bit number)
- 2 bytes: Client id (16-bit number, client-to-server messages only)
- 4 bytes: Payload hash (32-bit number, optional)
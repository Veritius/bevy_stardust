# Protocol
The protocol is not set in stone, and is liable to change at any time, though an attempt is made at maintaining compatibility.

Unless specified otherwise, the protocol uses the variable-length integers specified in RFC 9000.

# Datagrams
Unreliable-unordered and unreliable-sequenced messages are usually sent over datagrams wrapped in QUIC packets. Each datagram contains an additional header.

The header is first preceded by a variable-length integer indicating its **code**. The code identifies the purpose of the datagram, and the potential values are listed below.

| Code | Name                | Purpose                         |
|------|---------------------|---------------------------------|
| `0`  | `Stardust`          | Unreliable unordered messages   |
| `1`  | `StardustSequenced` | Unreliable unsequenced messages |

Some codes indicate additional data follows before the message.

<details>
<summary>Additional data for <code>Stardust</code></summary>

```
[varint] Channel identifier
```

</details>

<details>
<summary>Additional data for <code>StardustSequenced</code></summary>

```
[varint] Channel identifier
[seq16] Sequence identifier
```

</details>

# Streams
Reliable-unordered and reliable-unordered messages are sent over streams. Streams can also wrap datagrams to fragment them, if they are too large to send whole due to MTU constraints.

Streams are usually composed of chunks, using a length-prefix system. When a chunk is sent, it is preceded by its length, and the receiver will read that many bytes as a single chunk.

When streams are opened, the first chunk sent is the 'header' chunk. The header chunk always starts with the **code**, a variable-length integer. This identifies the purpose of the stream. The potential values are listed below.

| Code | Name              | Purpose            |
|------|-------------------|--------------------|
| `0`  | `Stardust`        | Stardust messages  |
| `1`  | `WrappedDatagram` | Contains datagrams |
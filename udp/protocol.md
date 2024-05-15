# Protocol
This is a document laying out the protocol works, mostly how packets appear on the wire. It's not hard and fast, and not definitely static - anything can change, just please update this document to improve your changes. This is also somewhat a record of why certain design choices were made.

## Handshake
**UNFINISHED**

The handshake is effectively a completely different protocol only run at the start of the connection, to establish the information necessary to proceed with the regular protocol.

The handshake is the only part of the protocol that is guaranteed to remain the same between crate minor versions. Steps are taken to ensure that a peer with an outdated version is detected and correctly informed of why their connection is being closed.

### Versioning data
This is done by including three values, sent by each peer in the handshake: the identifier, major version, and minor version. Note this has nothing to do with semantic versioning, and is **only** used in the handshake to ensure the peers are compatible with eachother.

The identifier is a static value unique to the crate, that is used to say what crate is being used to communicate. This simply checks that the same crate is being used to communicate. Since this crate can be forked, forks can change their identifier to signal incompatibility with other crates.

The major version is a value that tracks breaking changes to the post-handshake protocol, or the handshake protocol itself, since it is read first. This way, the overall protocol isn't locked into one design, and can be improved over time.

The minor version is a value that tracks non-breaking changes to the post-handshake protocol. The minor version is incremented every time this crate changes, and is also used to reject peers using a version of the crate with known issues.

These three values are hard-coded for the crate, but are also provided by the application at setup. This allows the exact same flexibility of updates to be applied to the application, while also preventing older, incompatible versions from connecting.

### Three-way handshake
The handshake occurs in three packets, very similarly to how TCP works, except over UDP. To explain this transaction, we'll call our two parties the **Initiator** and the **Listener**. The Initiator is the peer trying to join a multiplayer game, and the Listener is the peer listening for new connections for that game. Note that their roles become irrelevant once the handshake finishes.

#### Initiator hello
To begin, the Initiator sends an 'initiator hello' packet. This contains information like [reliability](#reliability) state, and [version](#versioning-data) data. Sending this packet implicitly says that the sender wants to create a virtual connection.

| Type  | Description                |
| ----- | -------------------------- |
| `u16` | Packet sequence identifier |
| `u64` | Transport identifier       |
| `u32` | Transport minor version    |
| `u32` | Transport major version    |
| `u64` | Application identifier     |
| `u32` | Application minor version  |
| `u32` | Application major version  |

The equivalent of this in TCP is the SYN packet sent by the client.

#### Listener response
In response, the Listener sends a 'listener response' packet. Like the 'initiator hello' packet, this contains reliability and version data. However, it also contains a [response](#response-codes) code, which informs the initiator whether or not the listener will continue with the handshake.

| Type  | Description                |
| ----- | -------------------------- |
| `u16` | Packet sequence identifier |
| `u16` | Response code              |
| `u64` | Transport identifier       |
| `u32` | Transport minor version    |
| `u32` | Transport major version    |
| `u64` | Application identifier     |
| `u32` | Application minor version  |
| `u32` | Application major version  |
| `u16` | Packet acknowledgement     |
| `u16` | Packet ack bitfield        |

If the response code signals an error or other rejection reason (non-zero), this is where the handshake ends. The listener may also include a human-readable disconnection reason, which will take up the rest of the packet.

The equivalent of this is in TCP is the SYN/ACK packet sent by the server.

#### Initiator response
Assuming the listener accepts the connection, the initiator will send the third and final packet in the handshake protocol. This is the 'initiator response' where there can be two outcomes: the connection becomes fully established, or the connection is terminated.

| Type  | Description                |
| ----- | -------------------------- |
| `u16` | Packet sequence identifier |
| `u16` | Response code              |
| `u16` | Packet acknowledgement     |
| `u16` | Packet ack bitfield        |

If the initiator decides that the connection should be closed, such as reading the listener response and seeing an incompatible version value, it can terminate the connection here.

If the initiator decides the connection should continue, it 'commits' and sends a 'continue' response code. At this point, we're done - the handshake is finished, and both peers transition into the established protocol.

The equivalent of this in TCP is the SYN packet sent by the client.

### Response codes
The following response codes will be the same for all minor versions of this crate, to ensure compatibility and good error messages.

| Int  | Description                            |
| ---- | -------------------------------------- |
| `0`  | No error, continue                     |
| `1`  | Unspecified error                      |
| `2`  | Malformed packet                       |
| `3`  | Invalid response code                  |
| `4`  | Incompatible transport layer           |
| `5`  | Incompatible transport major version   |
| `6`  | Incompatible transport minor version   |

These response codes may or may not change across a crate version, but if they do, they'll be marked as a breaking change.
| Int  | Description                            |
| ---- | -------------------------------------- |
| `7`  | Incompatible application identifier    |
| `8`  | Incompatible application major version |
| `9`  | Incompatible application minor version |
| `10` | Listener not accepting new connections |
| `11` | Connection terminated by application   |

## Established
### Header
All packets have an additional protocol header on top of the UDP header, to store extra data about the packet and reliability and such.

This is in reading order. For example, the packet flags will be read before the packet sequence id. The logic runs similarly to as you read this document.

```
[u8] Packet flags
```

The packet flags value changes the meaning of the next few bytes. This is what each bit means.

| Bit | Significance       |
| --- | ------------------ |
| `0` | Packet is reliable |
| `1` | Unassigned         |
| `2` | Unassigned         |
| `3` | Unassigned         |
| `4` | Unassigned         |
| `5` | Unassigned         |
| `6` | Unassigned         |
| `7` | Unassigned         |

If the packet is reliable (bit `0` is high), the packet is given a sequence id. Whether this field is present is only controlled by the flag: other fields will still appear.
```
[u16] Packet sequence id
```

The sequence ID of the last packet that the peer had seen at the time of sending. See the [reliability](#reliability) section for more information on how this whole system works.
```
[u16] Acknowledgement sequence
```

The ack bitfield is closely connected to the acknowledgement value, but the length is chosen at runtime, so it's represented here as `uX`. The bitfield can be `1` to `16` bytes long.
```
[uX] Acknowledgement bitfield
```

### Frames
After the header, the packet is composed of **frames.** These are individual message items within the packet.

```
[u8] Frame flags
```

The packet flags value changes the meaning of the next few bytes. This is what each bit means.

| Bit | Significance        |
| --- | ------------------- |
| `0` | Frame is ordered    |
| `1` | Frame is identified |
| `2` | Unassigned          |
| `3` | Unassigned          |
| `4` | Unassigned          |
| `5` | Unassigned          |
| `6` | Unassigned          |
| `7` | Unassigned          |

If the frame is identified (bit `1` is high), a varint will be read which controls the channel the message goes to. This is currently only used for Stardust message frames.

```
[uX] Id varint
```

If the frame is ordered (bit `0` is high), a `u16` with its sequence value will be read.

```
[u16] Ordering id
```

***

The frame type determines the purpose of the packet, and is used to have UDP control data and Stardust messages in the same packet.

```
[u8] Frame type
```

| Value | Variant  |
| ----- | -------- |
| `0`   | Control  |
| `1`   | Stardust |

Next is the length of the frame's payload. This is a varint of any length.

```
[uX] Length varint
```

Based on the length, the next N bytes will be read as a payload.

```
[uX] Payload
```
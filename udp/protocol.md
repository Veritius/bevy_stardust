# Protocol
This is a document laying out the protocol works, mostly how packets appear on the wire. It's not hard and fast, and not definitely static - anything can change, just please update this document to improve your changes. This is also somewhat a record of why certain design choices were made.

## Handshake
The handshake is a three-step handshake, similar to TCP, but with additional user-defined data. The two peers are named the **Initiator** (the peer trying to join) and the **Listener** (the peer waiting for clients).

TODO

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


### Reliability
TODO
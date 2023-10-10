# Using the transport layer

# Implementation details
## Forming a connection
Say we have two peers, A and B. A wants to create a connection with B.
To start, B must be listening for new connections. **TODO explain how**

A sends a message to B, starting with three bytes of zeroes. This is a 'zero message' and is used by the transport layer for creating connections. Zero messages are never used anywhere else.

The rest of the message is JSON, and looks like this.
```jsonc
{
    "transport": "udp-0.2.0",
    "request": "join",
    "protocol": "42B2EC801C40258A"
}
```
The `version` field is used by the transport layer to check compatibility between connection attempts. This is largely irrelevant to a user, and is handled entirely by the transport layer.

The `request` field defines what the purpose of the zero message is. In this case, it's A requesting to join B. There is currently no case other than `join` - other cases may be added in future. This is primarily used for future-proofing and defeating IP-spoofed amplification DoS attacks.

The `protocol` field contains a 64-bit unique hexadecimal number from the `UniqueNetworkHash` resource, created when building the `App`. This prevents peers with different protocol and plugin setups from joining eachother, negating the need for an expensive step where each peer exchanges their protocol data (or something like that).

***

B will see this zero message, and respond with something similar, headed by three bytes of zero once again.

In the case of B accepting A, they will send this JSON message.
```jsonc
{
    "response": "accepted",
    "port": 12345
}
```
The `port` field is the new port value that will be used to connect to the peer. When A receives this, they will store their IP and this new port value to communicate with them.

If A sends a packet that:
- is too small to be usable
- has invalid Unicode data
- is not a valid JSON table
- the `transport` field can't be parsed
- has no `request` field
- has other unexpected JSON data

it will discard the packet without informing A.

***

Alternatively, if B rejects A, they will send this:
```jsonc
{
    "response": "denied",
    "reason": "xyz"
}
```

`reason` has many possible values, listed below. More may be added in future.
- `unspecified` - No reason given towards the denial.
- `transport` - B doesn't support A's transport layer.
    - Has an extra field, `version`, containing the accepted versions.
- `protocol` - A's and B's protocol values differ.
    - Has an extra field, `requires`, containing B's protocol value.
- `closed` - B is closed to new connections.
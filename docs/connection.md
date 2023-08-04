# Stardust connection process
This process runs when a client tries to connect to a server.
This only applies to the default Stardust transport layers, ie UDP.

Transport layers from other crates may have their own process.

## UDP
UDP connection initially happens over TCP by transmitting JSON.

The JSON table **must** contain a key called `code`, with a number as the value.

<details>
<summary>All code values and their meanings</summary>

- `100` REQUEST_CONNECTION
    - Used by the client to request to join a remote server.
- `110` CONNECTION_ACKNOWLEDGED
    - Sent by the server to a client after `100` to acknowledge that they are being processed.
- `111` CONNECTION_HEARTBEAT
    - Sent by the server or client to check the other is still alive.
- `112` CONNECTION_HEARTBEAT_RESPONSE
    - Sent by the other party after `111` to confirm they are still active.
- `113` CONNECTION_ACCEPTED
    - The server has agreed to let the client join.
    - Responds with a `udp_port` field that is used for further UDP communication.
    - The TCP connection is closed after this.
- `120` CONNECTION_DENIED_BAD_PROTOCOL
    - The protocol ID of the client does not match the server.
- `121` CONNECTION_DENIED_NO_REASON
    - Sent by the server to a client after `100` to deny their connection.
- `122` CONNECTION_DENIED_FULL
    - Sent by the server to a client after `100` to deny their connection, indicating that there is no space for them to join.
- `123` CONNECTION_DENIED_CUSTOM
    - Arbitrary reason for denial. Response will have a `reason` field that should be shown to the client.
- `200` INVALID_PACKET_JSON
    - Packet could not be parsed into JSON.
- `201` INVALID_PACKET_CODE
    - Code didn't exist or didn't have the necessary fields.
</details>

<details>
<summary>Example connection exchange</summary>

```json
// From the client
{
    // This code value is present in all messages
    // Client requests to join
    "code": 100,

    // 64-bit protocol ID value, in hexadecimal form
    // This is used to compare against the server's protocol ID to prevent different games from joining
    // The value used in this doc is not from a real game
    "protocol": "41CAFA8289B97A2"
}

// From the server
{
    // Acknowledgment of connection, server is waiting for something before connecting
    "code": 101
}

// Server does some processing

// Connection accepted
{
    "code": 113
}
```
</details>
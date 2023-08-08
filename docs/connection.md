# Stardust connection process
This process runs when a client tries to connect to a server.
This only applies to the default Stardust transport layers, ie UDP.

Transport layers from other crates may have their own process.

## UDP
UDP connection initially happens over TCP by transmitting JSON documents.

Once a TCP connection is established by the standard library, the client starts by sending the version of the UDP transport layer, and a unique value that is used to verify that Stardust is configured identically on both the server and client.

```json
{
    // The version of the UDP transport layer
    "layer_version": "0.1.0",
    // The client's unique protocol hash
    "protocol": "D7799D37A7A9B082"
}
```

The server will then check the `version` and `pid` values, and send an appropriate response.

```json
// The layer version value is invalid
{ "response": "wrong_layer_version", "range": "=0.1.0" }
```
```json
// The pid value is incorrect
// This pid value is random and exists only for example's sake
{ "response": "wrong_pid", "srv_pid": "D7799D37A7A9B082" }
```
```json
// Too many players online
{ "response": "at_capacity" }
```
```json
// Something went wrong, the server requests the client to send the packet again
{ "response": "retry" }
```
```json
// Player is accepted, UDP port is provided.
{ "response": "accepted", "port": 12345 }
```
```json
// Player is denied, no reason given.
{ "response": "denied" }
```

After a response, the connection is closed.
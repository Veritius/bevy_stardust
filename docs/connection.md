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
    "version": "Stardust UDP v0.1.0",
    // The client's unique protocol hash
    "pid": "D7799D37A7A9B082"
}
```

The server will then check the `version` and `pid` values, and send an appropriate response.

```json
// The version value is invalid
{ "response": "invalid_version", "semver": "=0.1.0" }
```
```json
// The pid value is incorrect
{ "response": "wrong_pid", "srv_pid": "D7799D37A7A9B082" }
```
```json
// Something went wrong, errors are in the error list
{ "response": "error", "errors": [] }
```
```json
// Everything checks out
{ "response": "accepted" }
```

After a response, the connection is closed.
# Using Stardust

## Terminology
- **Channels:** A system of compartmentalising network messages, primarily utilised with Rust's type system.
- **Transport layer:** Code that deals with sending octet strings over the internet, using protocols like UDP or WebRTC.
- **Octet string:** An arbitrary-length array of octets (bytes) that is used by the transport layer to transmit information over a given medium (usually the Internet)

## Setup
### Plugins
Clients need the following plugins to function:
- `StardustSharedPlugin`
- `StardustClientPlugin`
- A transport layer plugin. If in doubt, you can use the included UDP transport layer, called `ClientUdpTransportPlugin`

Servers need the following plugins to function:
- `StardustSharedPlugin`
- `StardustServerPlugin`
- A transport layer plugin. If in doubt, you can use the included UDP transport layer, called `ServerUdpTransportPlugin`

Adding these plugins is required before any further setup, like adding channels.

### Channels
To send a network message requires you to set up **channels**. This can be done while creating the `App`, with the `register_channel` function.

`register_channel` takes a generic type implementing `Channel`, a trait that is automatically implemented for any type implementing `Debug + Send + Sync + 'static`, as well as the `ChannelConfig` object, and an `impl Bundle`.

The reason for taking a bundle as an argument, is that `register_channel` creates a new entity to store configuration information. This is used so channels can be composition-based, and external transport layers can add their own behaviors.

Usage example:
```rs
#[derive(Debug)]
struct MyChannel;

let config = ChannelConfig { direction: ChannelDirection::Bidirectional };
let components = ReliableChannel;
app.register_channel::<MyChannel>(config, components);
```

## Writing systems
Systems that use networking functionality must be in the following two schedules: `ReadOctetStrings` and `WriteOctetStrings`.

TODO
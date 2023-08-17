# Using Stardust

## Terminology
- **Channels:** A system of compartmentalising network messages, primarily utilised with Rust's type system.
- **Transport layer:** Code that deals with sending octet strings over the internet, using protocols like UDP or WebRTC.
- **Octet string:** An arbitrary-length array of octets (bytes) that is used by the transport layer to transmit information over a given medium (usually the Internet)

## Setup
Please note that for all setup, the order of operations does matter. Registering channel A before channel B on the server, and vice versa on the client, will prevent the client from joining the server.

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
Channels are used to compartmentalise network messages for better parallelism and to better suit the ECS design. They are primarily accessed using generic type APIs, though untyped APIs exist for use by transport layers. The generic type you use when creating the channel can then be used to refer back to it as a system parameter.

Creating a channel is done with the `register_channel` function, which can be used on the `App`. `register_channel` has one generic type for your channel, and two arguments to configure it. The first argument is the same type for every channel, a `ChannelConfig`. The second argument is an `impl Bundle`, which is used to add components to an entity storing configuration data.

The second argument is a `Bundle` so components from external crates can be added if need be. These components are used by the transport layer to enable functionality like reliability and ordering. Note that adding a component does not ensure that the transport layer you have chosen does support that functionality.

Stardust has the following components you can add to configure a channel.
> Note: Items marked with 🟡 means that the related functionality is not yet present.
- 🟡 `OrderedChannel`
    - Enables ordering. Packets will be sorted so when read, they are in the order they are sent.
    - This by itself does not guarantee that the packets arrive at all, just that they're in order.
- 🟡 `ReliableChannel`
    - Enables reliability. All packets sent over this channel are requested to be resent if they don't arrive.
    - This is not a fast process. With high latency, a package can arrive several seconds after it's sent.
    - With `OrderedChannel`, this can block all messages on the channel from being read until every packet is retrieved.
- 🟡 `ChannelLatestness`
    - Discards messages that are a certain amount of ticks old.
- 🟡 `FragmentedChannel`
    - If an octet string is too large to send in one packet, it will be broken into multiple packets for transmission.
    - Fragmentation channels implement their own kind of reliability, for single messages.
- 🟡 `CompressedChannel`
    - Octet strings in this channel will be run through a compression algorithm before transmission.
    - This can be useful with `FragmentedChannel` if your message is really, really big.

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
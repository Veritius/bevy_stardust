# Using Stardust

## Setting up the protocol
You must mutably access the `ProtocolBuilder` resource to modify the protocol. This can only be done when setting up the `App`.

### Adding channels
Channels can be added with `builder.add_channel<T>(config)` or `app.add_net_channel<T>(config)`. Your channel will be accessed by `T` which must be implement `Channel` (auto-impl exists). You can easily do this by making a unit struct and using `#[derive(Debug)]`.

Both channel registration methods take a `ChannelConfig`, in which you can configure features you want your channel to have. They are as follows:
- **messages_per_tick_server** - Amount of memory to pre-allocate for incoming messages. Smaller values use less memory, but overrunning the allocated amount can be very slow.
- **messages_per_tick_client** - The same, but for the client.

The following options exist, but are not functional as of yet.
- **direction** - Whether the channel should ignore messages coming from a certain direction.
- **reliability** - Whether the channel should ensure that all packets from the channel arrive.
- **latestness** - Discards messages from older ticks, useful for information that must be about the latest state of the game.
- **ordering** - Whether the channel should ensure that messages will be read by systems in the order they are sent.
- **error_checking** - Adds extra information to try and ensure that the message payload is correct.
- **fragmentation** - Whether messages over the maximum data limit (currently 1,500 bytes) should be broken into multiple packets for transmission.
- **compression** - Whether or not the message data is compressed. Useful with channel fragmentation for very large messages.

### Replicating components
You can replicate components by adding a special Plugin to the app. There are two, `ReplicateComponentPluginBitstream` and `ReplicateComponentPluginReflected`. The Bitstream plugin is used for components that implement the `ManualBitSerialisation` trait, and the Reflected plugin is used for components that implement Bevy's `Reflect` trait. If possible, use the bitstream plugin, as the reflected plugin is extremely inefficient to serialise the component information. 

### Finishing up
Finally, use the `gen_protocol_id` function on the `App` to create your protocol ID. This will allow the protocol ID to change if Stardust's internal networking code changes, preventing issues. You must still change the information passed to the function if you change the plugins.
Alternatively, manually set it with `set_id` on the `ProtocolBuilder`. This keeps your protocol ID under your control, but can cause problems.

The `ProtocolBuilder` will be removed in the `finish` step for Bevy plugins, and an unchangeable `Protocol` object will be added to the `World` for your systems to use.

## Adding to Stardust with Bevy plugins
Using the `expose_internals` feature flag lets you view some of the nuts and bolts of Stardust, useful if you're looking to optimise your network code.

**Put anything and everything that changes the protocol in a shared crate, or another solution that ensures that the protocol is identical on both the client and server.**
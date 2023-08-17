<h1><p align="center">✨ bevy_stardust</p></h1>
Stardust is a batteries-included networking crate built for Bevy. Stardust intends to make networking easy, but lets you do the hard stuff when you want to.
<br></br>

![License badge](https://img.shields.io/github/license/veritius/bevy_stardust)
![Bevy version badge](https://img.shields.io/badge/bevy-0.11-blue?color=blue)


## Features
- Tightly integrated with Bevy ECS - everything is part of the `World` and `App`.
- Effortlessly write networked code as part of your regular Bevy systems.
- Automatically compartmentalised network messages, separated into 'channels' defined with Bevy components.
- Runs in parallel - Stardust network code is built off the back of Bevy's scheduler, so your systems run perfectly in parallel.
- Plugins can effortlessly add their own network code without any changes on your side.
- Use any transport layer to send messages over the internet, including UDP, WebRTC, even [signal flags](https://en.wikipedia.org/wiki/International_maritime_signal_flags) - without changing any of your systems.
- Replicate components, even those from other crates, with a single line of code.
- Control replication on a per-entity basis with components and bundles.

*Note: While you can use any transport layer, Stardust by itself only supports native UDP.*

### Planned features
- Reliability
- Ordering
- Fragmentation
- Error checking
- Compression
- Encryption
- Randomness
- Replication
- `bevy_mod_scripting` support

## Usage
*This applies to the UDP transport layer.*

```rs
// In a shared location

use bevy::prelude::*;
use bevy_stardust::shared::prelude::*;

#[derive(Debug, Reflect)]
struct MyChannel;
```
```rs
// On the client

use bevy::prelude::*;
use bevy_stardust::client::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(StardustClientPlugin);
    app.add_plugins(ClientUdpTransportPlugin);

    app.register_channel::<MyChannel>(ChannelConfig {
        direction: ChannelDirection::Bidirectional,
    }, ());

    app.add_systems(WriteOctetStrings, sender_system);
}

fn sender_system(
    connection: Res<State<RemoteConnectionStatus>>,
    mut writer: ChannelWriter<MyChannel>,
) {
    if !connection.connected() { return; }

    writer.send("Hello, world!".into_bytes()).unwrap();
}
```
```rs
// On the server

use bevy::prelude::*;
use bevy_stardust::server::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(StardustServerPlugin);
    app.add_plugins(ServerUdpTransportPlugin {
        address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        listen_port: 12345,
        active_ports: 12346..=12356,
    })

    app.register_channel::<MyChannel>(ChannelConfig {
        direction: ChannelDirection::Bidirectional,
    }, ());

    app.insert_resource(NetworkClientCap(64));

    app.add_systems(ReadOctetStrings, receiver_system);
}

fn receiver_system(
    reader: ChannelReader<MyChannel>,
) {
    for (client, messages) in reader.read_all() {
        for message in messages.0.iter() {
            let string = String::from_utf8_lossy(message.read());
            println!("{:?} said {:?}", client, &string);
        }
    }
}
```
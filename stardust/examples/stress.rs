//! Example that measures the performance of Stardust.

const TEST_DURATION: Duration = Duration::from_secs(30);
const REPEAT_AMOUNT: usize = 1024;

use std::time::{Duration, Instant};
use rand::{Rng, seq::SliceRandom};
use bevy::{prelude::*, app::SubApp};
use bevy_stardust::prelude::*;

use channels::*;

fn main() {
    let mut owner = App::new();
    owner.add_plugins(DefaultPlugins);

    owner.insert_sub_app("server", SubApp::new(server::server(), |_,_| {}));
    owner.insert_sub_app("client", SubApp::new(client::client(), |_,_| {}));

    let start = Instant::now();
    let mut iterations: usize = 0;

    loop {
        if Instant::now().duration_since(start) > TEST_DURATION { break; }

        owner.update();
        iterations += 1;
    }

    println!("Did {:?} iterations in {:?}", iterations, TEST_DURATION);
}

fn apply_shared_data(app: &mut App) {
    app.add_plugins(MinimalPlugins);

    app.register_channel::<UnorderedUnreliableChannel>(());
    app.register_channel::<OrderedUnreliableChannel>(OrderedChannel);
    app.register_channel::<UnorderedReliableChannel>(ReliableChannel);
    app.register_channel::<OrderedReliableChannel>((OrderedChannel, ReliableChannel));
}

/// The greek alphabet, used for random string generation.
pub static GREEK_ALPHABET: &'static [&'static str] = &[
    "Alpha", "Beta", "Gamma",
    "Delta", "Epsilon", "Zeta",
    "Eta", "Theta", "Iota",
    "Kappa", "Lambda", "Mu",
    "Nu", "Xi", "Omicron",
    "Pi", "Rho", "Sigma",
    "Tau", "Upsilon", "Phi",
    "Chi", "Psi", "Omega",
];

/// Generates a random string.
fn gen_random_string() -> String {
    let mut rng = rand::thread_rng();
    let mut string = String::new();

    let len = rng.gen_range(4..=12);
    
    let mut x = 0;
    while x <= len {
        let choice = GREEK_ALPHABET.choose(&mut rng).unwrap();
        let choice = if string.len() != 0 { format!(" {}", choice) } else { choice.to_string() };
        string.push_str(&choice);
        x += 1;
    }

    string
}

mod channels {
    use bevy::reflect::TypePath;

    #[derive(Debug, TypePath)]
    pub struct UnorderedUnreliableChannel;

    #[derive(Debug, TypePath)]
    pub struct OrderedUnreliableChannel;

    #[derive(Debug, TypePath)]
    pub struct UnorderedReliableChannel;

    #[derive(Debug, TypePath)]
    pub struct OrderedReliableChannel;
}

mod client {
    use bevy::prelude::*;
    use bevy_stardust::channels::id::Channel;
    use bevy_stardust::client::connection::RemoteConnectionStatus;
    use bevy_stardust::prelude::client::*;
    use bevy_stardust::scheduling::*;
    use bevy_stardust::setup::*;
    use bevy_stardust::transports::udp::prelude::*;
    use semver::Version;
    use semver::VersionReq;

    use crate::REPEAT_AMOUNT;
    use crate::channels::*;
    use crate::apply_shared_data;
    use crate::gen_random_string;

    pub(super) fn client() -> App {
        let mut app = App::new();

        app.add_plugins(StardustPlugin {
            version: Version::new(0, 0, 0),
            allows: VersionReq::STAR,
        });
        app.add_plugins(ClientUdpTransportPlugin);

        apply_shared_data(&mut app);

        app.add_systems(ReadOctetStrings, receive_random_data_system_client::<UnorderedUnreliableChannel>);
        app.add_systems(ReadOctetStrings, receive_random_data_system_client::<OrderedUnreliableChannel>);
        app.add_systems(ReadOctetStrings, receive_random_data_system_client::<UnorderedReliableChannel>);
        app.add_systems(ReadOctetStrings, receive_random_data_system_client::<OrderedReliableChannel>);
        app.add_systems(Update, send_random_data_system_client::<UnorderedUnreliableChannel>);
        app.add_systems(Update, send_random_data_system_client::<OrderedUnreliableChannel>);
        app.add_systems(Update, send_random_data_system_client::<UnorderedReliableChannel>);
        app.add_systems(Update, send_random_data_system_client::<OrderedReliableChannel>);

        app.add_systems(Startup, |mut manager: UdpConnectionManager| {
            use std::net::*;
            let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
            manager.join(SocketAddr::new(ip, 12345));
        });
        
        app
    }

    fn send_random_data_system_client<T: Channel>(
        conn: Res<State<RemoteConnectionStatus>>,
        mut writer: ChannelWriter<T>,
    ) {
        if !conn.connected() { return; }
    
        let string = gen_random_string();

        for _ in 0..REPEAT_AMOUNT {
            let _ = writer.send(string.clone().into_bytes());
        }
    }
    
    fn receive_random_data_system_client<T: Channel>(
        reader: ChannelReader<T>,
    ) {
        let payloads = reader.read_from_server();
        let Ok(Some(payloads)) = payloads else { return };
    
        for payload in payloads.0.iter() {
            let _ = payload.read();
        }
    }
}

mod server {
    use bevy::prelude::*;
    use bevy_stardust::channels::id::Channel;
    use bevy_stardust::channels::outgoing::SendTarget;
    use bevy_stardust::scheduling::*;
    use bevy_stardust::prelude::server::*;
    use bevy_stardust::setup::*;
    use bevy_stardust::transports::udp::prelude::ServerUdpTransportPlugin;
    use semver::Version;
    use semver::VersionReq;

    use crate::REPEAT_AMOUNT;
    use crate::channels::*;
    use crate::apply_shared_data;
    use crate::gen_random_string;

    pub(super) fn server() -> App {
        let mut app = App::new();

        app.add_plugins(StardustPlugin {
            version: Version::new(0, 0, 0),
            allows: VersionReq::STAR,
        });
        app.add_plugins(ServerUdpTransportPlugin {
            address: None,
            listen_port: 12345,
            active_ports: (12346..=12356).collect::<Vec<_>>(),
        });

        apply_shared_data(&mut app);

        // Add our sending and receiving systems
        app.add_systems(ReadOctetStrings, receive_random_data_system_server::<UnorderedUnreliableChannel>);
        app.add_systems(ReadOctetStrings, receive_random_data_system_server::<OrderedUnreliableChannel>);
        app.add_systems(ReadOctetStrings, receive_random_data_system_server::<UnorderedReliableChannel>);
        app.add_systems(ReadOctetStrings, receive_random_data_system_server::<OrderedReliableChannel>);
        app.add_systems(Update, send_random_data_system_server::<UnorderedUnreliableChannel>);
        app.add_systems(Update, send_random_data_system_server::<OrderedUnreliableChannel>);
        app.add_systems(Update, send_random_data_system_server::<UnorderedReliableChannel>);
        app.add_systems(Update, send_random_data_system_server::<OrderedReliableChannel>);


        // Configure the server
        app.insert_resource(NetworkClientCap(64));

        app
    }

    fn send_random_data_system_server<T: Channel>(
        clients: Query<(), With<Client>>,
        mut writer: ChannelWriter<T>,
    ) {
        if clients.is_empty() { return; }
        
        let string = gen_random_string();

        for _ in 0..REPEAT_AMOUNT {
            let _ = writer.send(SendTarget::Broadcast, string.clone().into_bytes());
        }
    }
    
    fn receive_random_data_system_server<T: Channel>(
        reader: ChannelReader<T>,
    ) {
        let iter = reader.read_all();
        for (_, messages) in iter {
            for message in messages.0.iter() {
                let _ = message.read();
            }
        }
    }
}
use std::sync::Arc;
use bevy::{app::{AppLabel, SubApp}, ecs::schedule::ExecutorKind, prelude::*};
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;
use rustls::{Certificate, PrivateKey, RootCertStore};

const SERVER_ALT_NAME: &str = "www.icann.org";
const SERVER_ADDRESS: &str = "127.0.0.1:12344";
const CLIENT_ADDRESS: &str = "127.0.0.1:12345";

#[derive(TypePath)]
struct MyMessage;

fn main() {
    // Create a self-signed certificate for this example
    // It's not really distributed by ICANN but we need an alt name here
    let selfsigned = rcgen::generate_simple_self_signed(vec![String::from(SERVER_ALT_NAME)]).unwrap();
    let cert = rustls::Certificate(selfsigned.serialize_der().unwrap());
    let key = rustls::PrivateKey(selfsigned.serialize_private_key_der());
    #[derive(Resource)] struct ServerPair(Certificate, PrivateKey);

    // Create root certificate store
    let mut root_certs = RootCertStore::empty();
    root_certs.add(&cert).unwrap();
    let root_certs = Arc::new(root_certs);
    #[derive(Resource)] struct RootCerts(Arc<RootCertStore>);

    // Client
    let mut client = setup_app();
    client.insert_resource(RootCerts(root_certs.clone()));
    client.add_systems(Startup, |certs: Res<RootCerts>, mut manager: QuicConnectionManager| {
        manager.open_client_endpoint(
            CLIENT_ADDRESS,
            certs.0.clone()
        ).unwrap();
    });
    client.add_systems(PostStartup, |endpoints: Query<Entity, With<QuicEndpoint>>, mut manager: QuicConnectionManager| {
        manager.try_connect_remote(endpoints.single(), SERVER_ADDRESS, SERVER_ALT_NAME).unwrap();
    });

    // Server
    let mut server = setup_app();
    server.insert_resource(RootCerts(root_certs.clone()));
    server.insert_resource(ServerPair(cert, key));
    server.add_systems(Startup, |certs: Res<RootCerts>, pair: Res<ServerPair>, mut manager: QuicConnectionManager| {
        manager.open_server_endpoint(
            SERVER_ADDRESS,
            certs.0.clone(),
            vec![pair.0.clone()],
            pair.1.clone()
        ).unwrap();
    });

    // Super-app to run our other apps
    #[derive(Debug, Clone, Hash, PartialEq, Eq, AppLabel)]
    enum AppLabel { Client, Server }
    let mut master = App::new();
    master.add_plugins(MinimalPlugins);
    master.insert_sub_app(AppLabel::Client, SubApp::new(client, |_,_| {}));
    master.insert_sub_app(AppLabel::Server, SubApp::new(server, |_,_| {}));

    // Run app loop
    loop { master.update(); }
}

fn setup_app() -> App {
    let mut app = App::new();
    app.edit_schedule(Main, |f| {
        f.set_executor_kind(ExecutorKind::SingleThreaded) ;
    });

    app.add_plugins(StardustPlugin);

    app.add_channel::<MyMessage>(ChannelConfiguration {
        reliable: ReliabilityGuarantee::Reliable,
        ordered: OrderingGuarantee::Ordered,
        fragmented: false,
        string_size: 0..=128,
    });

    app.add_plugins(QuicTransportPlugin {
        allow_self_signed: true,
        reliable_streams: 8,
        timeout_delay: 30,
    });

    app
}
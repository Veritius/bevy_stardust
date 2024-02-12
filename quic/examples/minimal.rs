use std::{sync::Arc, thread};
use bevy::{prelude::*, ecs::schedule::ExecutorKind};
use bevy_stardust::prelude::*;
use bevy_stardust_quic::*;
use rustls::RootCertStore;

const SERVER_ADDRESS: &str = "127.0.0.1:12344";
const CLIENT_ADDRESS: &str = "127.0.0.1:12345";

#[derive(TypePath)]
struct MyMessage;

fn main() {
    // Create a self-signed certificate for this example
    // It's not really distributed by ICANN but we need an alt name here
    let selfsigned = rcgen::generate_simple_self_signed(vec![String::from("https://www.icann.org/")]).unwrap();
    let cert = rustls::Certificate(selfsigned.serialize_der().unwrap());
    let key = rustls::PrivateKey(selfsigned.serialize_private_key_der());

    // Create root certificate store
    let mut root_certs = RootCertStore::empty();
    root_certs.add(&cert).unwrap();
    let root_certs = Arc::new(root_certs);

    // Create apps
    let mut client = App::new();
    let mut server = App::new();

    todo!()
}

fn setup_app() -> App {
    let mut app = App::new();
    app.edit_schedule(Main, |f| {
        f.set_executor_kind(ExecutorKind::SingleThreaded) ;
    });

    app.add_plugins(MinimalPlugins);
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
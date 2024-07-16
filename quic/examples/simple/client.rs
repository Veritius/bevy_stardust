mod shared;

use std::net::{IpAddr, Ipv4Addr};
use bevy::prelude::*;
use bevy_stardust_quic::*;

fn main() {
    let mut app = shared::app();

    app.add_systems(Startup, |mut commands: Commands| {
        let endpoint = EndpointBuilder::client()
            .with_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED)).unwrap()
            .with_protos(shared::app_protos())
            .with_trust_anchors(shared::trust_anchors())
            .build().unwrap();

        commands.spawn(endpoint);
    });

    app.run();
}
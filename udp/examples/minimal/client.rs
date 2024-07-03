mod shared;
use shared::*;

use std::time::Duration;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_udp::prelude::*;

const DISCONNECT_TIME: Duration = Duration::from_secs(10);

fn main() {
    let mut app = setup_app();

    app.add_systems(Startup, |mut manager: UdpManager| {
        manager.open_endpoint_and_connect(UNSPECIFIED_SOCKET_ADDR, LISTENER_ADDRESS).unwrap();
    });

    app.add_systems(Update, |
        peers: Query<(Entity, &Peer)>,
        mut events: EventWriter<DisconnectPeerEvent>,
    | {
        for (id, comp) in peers.iter() {
            if comp.joined.elapsed() > DISCONNECT_TIME {
                events.send(DisconnectPeerEvent {
                    peer: id,
                    reason: DisconnectReason::Unspecified,
                    comment: None,
                    force: false,
                });
            }
        }
    });

    app.run();
}
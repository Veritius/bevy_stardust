use std::time::Duration;

use bevy::app::{AppLabel, RunMode, ScheduleRunnerPlugin, SubApp};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust::testing::transport::*;
use bevy_stardust_replicate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Event, Serialize, Deserialize)]
enum PlayerMoveEvent { Up, Down, Left, Right }

#[derive(Debug, Event, Serialize, Deserialize)]
struct PlayerAttackEvent;

fn main() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AppLabel)]
    enum SubappLabel { Server, Client }

    let mut server = App::new();
    let mut client = App::new();

    let (link_left, link_right) = pair();

    #[inline]
    fn spawn_peer(world: &mut World) -> EntityWorldMut {
        world.spawn((
            NetworkPeer::new(),
            NetworkMessages::<Incoming>::new(),
            NetworkMessages::<Outgoing>::new(),
        ))
    }

    spawn_peer(&mut server.world).insert((ReplicationPeer::new(Side::Server), link_left));
    spawn_peer(&mut client.world).insert((ReplicationPeer::new(Side::Client), link_right));

    for app in [&mut server, &mut client] {
        app.add_plugins((
            StardustPlugin,
            LinkTransportPlugin,
            CoreReplicationPlugin,
        ));

        app.add_plugins(EventRelayPlugin {
            serialisation: SerialisationFunctions::<PlayerMoveEvent>::serde(),
            reliability: ReliabilityGuarantee::Unreliable,
            ordering: OrderingGuarantee::Sequenced,
            message_priority: 64,
            phantom: Default::default(),
        });

        app.add_plugins(EventRelayPlugin {
            serialisation: SerialisationFunctions::<PlayerAttackEvent>::serde(),
            reliability: ReliabilityGuarantee::Reliable,
            ordering: OrderingGuarantee::Ordered,
            message_priority: 256,
            phantom: Default::default(),
        });

        // Manually invoke finish as this is a subapp.
        app.finish();
    }

    server.add_systems(Update, read_events_system);
    client.add_systems(Update, random_events_system);

    let mut core = App::new();
    core.add_plugins((
        ScheduleRunnerPlugin { run_mode: RunMode::Loop { wait: Some(Duration::from_millis(500)) }},
        LogPlugin::default()
    ));

    core.insert_sub_app(SubappLabel::Server, SubApp::new(server, |_,_| {}));
    core.insert_sub_app(SubappLabel::Client, SubApp::new(client, |_,_| {}));

    core.run();
}

fn read_events_system(
    mut movements: NetEventReader<PlayerMoveEvent>,
    mut attacks: NetEventReader<PlayerAttackEvent>,
) {
    for net_event in movements.read() {
        info!("{:?} moved {:?}", net_event.origin, net_event.event);
    }

    for net_event in attacks.read() {
        info!("{:?} attacked!", net_event.origin);
    }
}

fn random_events_system(
    mut movements: EventWriter<PlayerMoveEvent>,
    mut attacks: EventWriter<PlayerAttackEvent>,
) {
    // Pick a random direction to move
    let movement = match fastrand::u8(..4) {
        0 => PlayerMoveEvent::Up,
        1 => PlayerMoveEvent::Down,
        2 => PlayerMoveEvent::Left,
        3 => PlayerMoveEvent::Right,
        _ => unreachable!(),
    };

    // Log to console for demonstration
    info!("Moving {movement:?}");

    // Send the event to the peer
    movements.send(movement);

    if fastrand::f32() > 0.7 {
        info!("Attacking!");
        attacks.send(PlayerAttackEvent);
    }
}
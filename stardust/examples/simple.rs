use std::any::TypeId;

use bevy::app::{AppLabel, SubApp, ScheduleRunnerPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust::testing::transport::*;

#[derive(TypePath)]
struct MyChannelA;

#[derive(TypePath)]
struct MyChannelB;

#[derive(TypePath)]
struct MyChannelC;

#[derive(Resource)]
struct AppName(&'static str);

fn main() {
    let mut left = App::new();
    left.insert_resource(AppName("Left"));

    let mut right = App::new();
    right.insert_resource(AppName("Right"));

    let (link_left, link_right) = pair();
    left.world.spawn((NetworkPeer::new(), NetworkMessages::<Incoming>::new(), NetworkMessages::<Outgoing>::new(), link_left));
    right.world.spawn((NetworkPeer::new(), NetworkMessages::<Incoming>::new(), NetworkMessages::<Outgoing>::new(), link_right));

    for app in [&mut left, &mut right] {
        app.add_plugins((StardustPlugin, LinkTransportPlugin));

        let config = ChannelConfiguration {
            consistency: Consistency::ReliableOrdered,
            priority: 0,
        };

        app.add_channel::<MyChannelA>(config.clone());
        app.add_channel::<MyChannelB>(config.clone());
        app.add_channel::<MyChannelC>(config.clone());

        app.add_systems(Update, (
            read_system,
            write_system::<MyChannelA>,
            write_system::<MyChannelB>,
            write_system::<MyChannelC>,
        ));

        // Manually invoke finish as this is a subapp.
        app.finish();
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AppLabel)]
    enum SubappLabel { Left, Right }

    let mut core = App::new();
    core.add_plugins((ScheduleRunnerPlugin::default(), LogPlugin::default()));
    core.insert_sub_app(SubappLabel::Left, SubApp::new(left, |_,_| {}));
    core.insert_sub_app(SubappLabel::Right, SubApp::new(right, |_,_| {}));

    core.run();
}

fn read_system(
    name: Res<AppName>,
    query: Query<&NetworkMessages<Incoming>, With<NetworkPeer>>,
) {
    for incoming in query.iter() {
        for (channel, queues) in incoming {
            for payload in queues {
                info!("{}: Received a message from a peer on channel {channel:?}: {payload:?}", name.0);
            }
        }
    }
}

fn write_system<C: Channel>(
    name: Res<AppName>,
    registry: Res<ChannelRegistry>,
    mut query: Query<&mut NetworkMessages<Outgoing>, With<NetworkPeer>>,
) {
    for mut outgoing in query.iter_mut() {
        let rand = fastrand::u128(..);
        let bytes = Bytes::copy_from_slice(&rand.to_be_bytes()[..]);

        info!("{}: Sent a message to a peer: {bytes:?}", name.0);
        outgoing.push(registry.channel_id(TypeId::of::<C>()).unwrap(), bytes);
    }
}
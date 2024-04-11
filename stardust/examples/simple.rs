use std::any::TypeId;

use bevy::app::{AppLabel, SubApp, ScheduleRunnerPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust::testing::transport::*;

struct MyChannelA;
struct MyChannelB;
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
            reliable: ReliabilityGuarantee::Reliable,
            ordered: OrderingGuarantee::Ordered,
            fragmented: false,
            priority: 0,
        };

        app.add_channel::<MyChannelA>(config.clone());
        app.add_channel::<MyChannelB>(config.clone());
        app.add_channel::<MyChannelC>(config.clone());

        app.add_systems(Update, (
            rw_system::<MyChannelA>,
            rw_system::<MyChannelB>,
            rw_system::<MyChannelC>,
        ));
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, AppLabel)]
    enum SubappLabel { Left, Right }

    let mut core = App::new();
    core.add_plugins((ScheduleRunnerPlugin::default(), LogPlugin::default()));
    core.insert_sub_app(SubappLabel::Left, SubApp::new(left, |_,_| {}));
    core.insert_sub_app(SubappLabel::Right, SubApp::new(right, |_,_| {}));

    core.run();
}

fn rw_system<C: Channel>(
    name: Res<AppName>,
    registry: Res<ChannelRegistry>,
    mut query: Query<(&NetworkMessages<Incoming>, &mut NetworkMessages<Outgoing>), With<NetworkPeer>>,
) {
    for (incoming, mut outgoing) in query.iter_mut() {
        for (channel, queues) in incoming.all_queues() {
            for payload in queues {
                info!("{}: Received a message from a peer on channel {channel:?}: {payload:?}", name.0);
            }
        }

        let rand = fastrand::u128(..);
        let bytes = Bytes::copy_from_slice(&rand.to_be_bytes()[..]);

        info!("{}: Sent a message to a peer: {bytes:?}", name.0);
        outgoing.push(registry.channel_id(TypeId::of::<C>()).unwrap(), bytes);
    }
}
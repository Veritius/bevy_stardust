use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quinn::QuinnPlugin;

pub const CERTIFICATE: &str = include_str!("../certs/certificate.crt");

pub fn setup(app: &mut App) {
    app.add_plugins((
        DefaultPlugins,
        StardustPlugin,
        QuinnPlugin,
    ));

    app.add_event::<MovementEvent>();

    app.add_channel::<MovementEvent>(ChannelConfiguration {
        consistency: ChannelConsistency::UnreliableSequenced,
        priority: 32,
    });
}

#[derive(Event)]
pub struct MovementEvent {
    pub direction: Vec2,
}
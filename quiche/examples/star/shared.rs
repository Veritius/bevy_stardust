use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quiche::QuichePlugin;

// NOTE: It is very, very, very bad practice to compile-in certificates.
// This is only done for the sake of simplicity. In reality, you should
// get private keys and certificates from files.
pub const CERTIFICATE: &str = include_str!("../certs/certificate.crt");

pub fn setup(app: &mut App) {
    app.add_plugins((
        DefaultPlugins,
        StardustPlugin,
        QuichePlugin,
    ));

    app.add_channel::<MovementEvent>(ChannelConfiguration {
        consistency: MessageConsistency::UnreliableSequenced,
        priority: 32,
    });

    app.add_event::<MovementEvent>();
}

#[derive(Debug, Event)]
pub struct MovementEvent {
    degree: Vec2,
}
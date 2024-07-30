use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quinn::QuinnPlugin;

pub fn setup(app: &mut App) {
    app.add_plugins((
        DefaultPlugins,
        StardustPlugin,
        QuinnPlugin,
    ));

    app.add_channel::<MovementEvent>(ChannelConfiguration {
        consistency: ChannelConsistency::UnreliableSequenced,
        priority: 32,
    });

    app.add_event::<MovementEvent>();
}

#[derive(Debug, Event)]
pub struct MovementEvent {
    degree: Vec2,
}
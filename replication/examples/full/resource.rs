use bevy::prelude::*;
use bevy_stardust_replicate::prelude::*;
use serde::{Serialize, Deserialize};

pub(super) fn setup_resources(app: &mut App) {
    app.add_plugins(ResourceReplicationPlugin {
        serialisation: SerialisationFunctions::<MoveSpeedMultiplier>::serde(),
        message_priority: 1024,
    });

    app.insert_resource(MoveSpeedMultiplier { value: 1.0 });
    let z = NetChangeTracking::<MoveSpeedMultiplier>::from_world(&mut app.world);
    app.insert_resource(z);

    app.add_systems(Startup, spawn_resource_text_system);

    app.add_systems(Update, (
        adjust_resource_system,
        update_resource_text_system,
    ).chain());
}

#[derive(TypePath, Resource, Serialize, Deserialize)]
pub(crate) struct MoveSpeedMultiplier {
    pub value: f32,
}

fn adjust_resource_system(
    buttons: Res<ButtonInput<KeyCode>>,
    mut res: ResMut<MoveSpeedMultiplier>,
    time: Res<Time>,
) {
    if buttons.pressed(KeyCode::ArrowUp) {
        res.value += 1.0 * time.delta_seconds();
    }

    if buttons.pressed(KeyCode::ArrowDown) {
        res.value -= 1.0 * time.delta_seconds();
    }
}

#[derive(Component)]
struct ResourceDisplay;

fn spawn_resource_text_system(
    mut commands: Commands,
) {
    commands.spawn((ResourceDisplay, TextBundle {
        text: Text::from_sections([
            TextSection::new(
                "\n",
                TextStyle { font_size: 20.0, ..default() }),
            TextSection::new(
                "Use ArrowUp to increase and ArrowDown to decrease",
                TextStyle { font_size: 16.0, ..default()}),
        ]),
        style: Style {
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            ..default()
        },
        ..default()
    }));
}

fn update_resource_text_system(
    res: NetRes<MoveSpeedMultiplier>,
    mut query: Query<&mut Text, With<ResourceDisplay>>,
) {
    if !res.is_changed() { return; }

    let mut text = query.single_mut();

    let icbr = res.is_changed_by_replication();
    let icba = res.is_changed_by_application();

    let msg = match (icbr, icba) {
        (false, true) => "this application",
        (true, false) => "the replication plugin",
        (false, false) => "nobody",
        (true, true) => unreachable!(),
    };

    text.sections[0].value = format!("The current movement speed is {}\nLast changed by {msg}\n", res.value);
}
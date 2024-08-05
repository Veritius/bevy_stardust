mod shared;

use std::sync::Arc;
use bevy::prelude::*;
use bevy_stardust_quinn::Manager;
use quinn_proto::EndpointConfig;

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.add_systems(Startup, |mut manager: Manager| {
        manager.open_endpoint(
            Arc::new(EndpointConfig::default()),
            None,
            shared::WILDCARD_ADDRESS,
        ).unwrap();

        todo!()
    });

    app.run();
}
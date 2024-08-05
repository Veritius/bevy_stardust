mod shared;

use std::sync::Arc;
use bevy::prelude::*;
use bevy_stardust_quinn::Endpoints;
use quinn_proto::EndpointConfig;

fn main() {
    let mut app = App::new();

    shared::setup(&mut app);

    app.add_systems(Startup, |mut endpoints: Endpoints| {
        endpoints.create(
            Arc::new(EndpointConfig::default()),
            None,
            shared::WILDCARD_ADDRESS,
        );

        todo!()
    });

    app.run();
}
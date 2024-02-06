use bevy::prelude::*;
use quinn::Endpoint;

#[derive(Resource)]
pub(crate) struct Endpoints(pub Vec<Endpoint>);
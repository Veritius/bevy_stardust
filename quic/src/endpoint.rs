use std::sync::Exclusive;
use bevy::prelude::*;

#[derive(Resource)]
pub(crate) struct Endpoint(pub Exclusive<quinn_proto::Endpoint>);
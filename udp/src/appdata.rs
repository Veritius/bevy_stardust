use bevy_ecs::prelude::*;

pub const TRANSPORT_IDENTIFIER: u64 = 0x0;
pub const TRANSPORT_VERSION_MAJOR: u32 = 0x0;
pub const TRANSPORT_VERSION_MINOR: u32 = 0x0;

#[derive(Debug, Resource, Clone)]
pub(super) struct ApplicationContext {
    pub application_identifier: u64,
}
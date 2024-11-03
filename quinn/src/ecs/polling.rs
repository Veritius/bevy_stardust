use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct BackendExecutor {
    executor: crate::backend::executor::BackendExecutor,
}
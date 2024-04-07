use bevy::prelude::*;
use super::*;

pub(crate) fn clear_message_queue_system<D: DirectionType>(
    mut queues: Query<&mut NetworkMessages<D>, Changed<NetworkMessages<D>>>,
) {
    queues.par_iter_mut().for_each(|mut queue| {
        queue.clear();
    });
}
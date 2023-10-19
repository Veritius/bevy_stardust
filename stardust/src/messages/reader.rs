//! Systemparams for game systems to read messages.

use std::marker::PhantomData;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::Channel;

use super::incoming::NetworkMessageStorage;

/// Allows game systems to read messages sent over channel `T`.
#[derive(SystemParam)]
pub struct MessageReader<'w, 's, T: Channel> {
    incoming: Query<'w, 's, &'static NetworkMessageStorage>,
    phantom: PhantomData<T>,
}
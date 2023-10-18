//! Systemparams for game systems to read messages.

use std::marker::PhantomData;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::{prelude::Channel, channels::incoming::IncomingNetworkMessages};

/// Allows game systems to read messages sent over channel `T`.
#[derive(SystemParam)]
pub struct MessageReader<'w, 's, T: Channel> {
    incoming: Query<'w, 's, &'static IncomingNetworkMessages>,
    phantom: PhantomData<T>,
}
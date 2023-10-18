//! Systemparams for game systems to write messages.

use std::marker::PhantomData;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::{prelude::Channel, channels::outgoing::OutgoingNetworkMessages};

/// Allows game systems to send messages to peers on channel `T`.
#[derive(SystemParam)]
pub struct MessageWriter<'w, 's, T: Channel> {
    outgoing: ResMut<'w, OutgoingNetworkMessages<T>>,
    phantom: PhantomData<&'s ()>,
}
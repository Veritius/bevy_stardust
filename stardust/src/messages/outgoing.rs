//! Systemparams for transport layers to send messages from game systems.

use bevy::{prelude::*, ecs::system::SystemParam};

/// Allows transport layers to view all messages sent by game systems for transport.
#[derive(SystemParam)]
pub struct TransportOutgoingWriter {
    
}

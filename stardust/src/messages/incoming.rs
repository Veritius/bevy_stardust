//! Systemparams for transport layers to read and store messages for game systems.

use std::collections::BTreeMap;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::prelude::{ChannelId, OctetString};

/// Allows transport layers to store incoming messages on entities for game systems to read.
#[derive(SystemParam)]
pub struct TransportIncomingWriter {
    
}

impl TransportIncomingWriter {

}

/// Storage for network messages that have been directed to this entity.
#[derive(Component)]
pub(super) struct NetworkMessageStorage(BTreeMap<ChannelId, Vec<OctetString>>);
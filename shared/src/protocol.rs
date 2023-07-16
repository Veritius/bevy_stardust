use std::{collections::{hash_map::DefaultHasher, HashMap}, hash::Hasher, any::TypeId};
use bevy::prelude::Resource;
use crate::channel::{Channel, ChannelConfig};

const STARDUST_PROTOCOL_VERSION: [u8; 2] = 0_u16.to_be_bytes();

#[derive(Resource)]
pub struct Protocol {
    unique_id: u32,
    channel_ids: HashMap<TypeId, u16>,
    channels: Vec<ChannelConfig>
}

impl Protocol {
    /// Returns the unique ID of this Protocol object.
    fn unique_id(&self) -> u32 { self.unique_id }
}

pub struct ProtocolBuilder {
    unique_id: Option<u32>,
    channels: Vec<(TypeId, ChannelConfig)>,
}

impl ProtocolBuilder {
    /// Creates a new blank `ProtocolBuilder`
    fn new() -> Self {
        Self {
            unique_id: None,
            channels: vec![],
        }
    }

    /// Generates a unique protocol ID from the project name and an arbitrary version value.
    /// You *must* use this function before building the protocol.
    fn generate_id(&mut self, name: String, version: String) {
        let mut hasher = DefaultHasher::new();
        hasher.write(&STARDUST_PROTOCOL_VERSION);
        hasher.write(name.as_bytes());
        hasher.write(version.as_bytes());
        // Take the first 32 bits from the hash result as a protocol ID.
        let hash: [u8; 8] = hasher.finish().to_be_bytes();
        let crushed = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]);
        self.unique_id = Some(crushed);
    }

    /// Adds a channel to the protocol.
    fn add_channel<T: Channel>(&mut self, config: ChannelConfig) {
        let id = TypeId::of::<T>();
        self.channels.push((id, config))
    }

    /// Consumes this `ProtocolBuilder` and returns a `Protocol` for use in networking.
    #[must_use]
    fn build(self) -> Protocol {
        // Check ID exists
        if self.unique_id.is_none() { panic!("No unique protocol ID set! Did you use generate_id first?"); }

        // Add channels to maps
        let mut series_id: u16 = 0;
        let mut channel_ids = HashMap::new();
        let mut channels = Vec::with_capacity(self.channels.len());
        for (type_id, config) in self.channels {
            let id = series_id;
            channel_ids.insert(type_id, id);
            channels.push(config);

            // Increase series_id and panic on overflow
            series_id = series_id.checked_add(1)
                .unwrap_or_else(|| panic!("Protocol channel cap of 65536 exceeded!"));
        }

        Protocol {
            unique_id: self.unique_id.unwrap(),
            channel_ids,
            channels,
        }
    }
}
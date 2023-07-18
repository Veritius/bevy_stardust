#[allow(deprecated)]
use std::hash::SipHasher;
use std::{any::TypeId, collections::BTreeMap, hash::Hasher};
use bevy::prelude::{App, Resource};
use crate::channel::{Channel, ChannelConfig, ChannelId, ChannelEncryption};

/// Maximum packet length that can be sent/received before fragmentation.
pub const MAX_PACKET_LENGTH: u16 = 1500;
/// Unique value for the Stardust protocol. 
const STARDUST_PROTOCOL_VERSION: u16 = 0;

/// The shared agreement between the client and server used to transport information.
/// 
/// This **must** be identical on both the client and server. The best way to achieve this is putting it in a shared location, ie another crate.
#[derive(Resource)]
pub struct Protocol {
    unique_id: u32,

    channel_types: BTreeMap<TypeId, ChannelId>,
    channels: Vec<(TypeId, ChannelConfig)>,
}

impl Protocol {
    /// Returns `true` if any channels have cryptographic features enabled.
    pub(crate) fn any_encrypted(&self) -> bool {
        for (_, config) in &self.channels {
            if config.encryption != ChannelEncryption::Disabled {
                return true;
            }
        }
        return false
    }

    /// Returns the Protocol ID
    pub fn id(&self) -> u32 {
        self.unique_id
    }
}

#[derive(Resource, Default)]
pub struct ProtocolBuilder {
    protocol_id: u32,
    channels: Vec<(TypeId, ChannelConfig)>,
}

impl ProtocolBuilder {
    pub fn set_id(&mut self, id: u32) { self.protocol_id = id; }

    pub fn add_channel<T: Channel>(&mut self, config: ChannelConfig) {
        let this = TypeId::of::<T>();
        for (other, _) in self.channels.iter() {
            // Prevent channels being added twice
            if this == *other { panic!("Channel added twice: {:?}", this); }
        }
        self.channels.push((this, config));
    }

    /// Consumes the `ProtocolBuilder` and returns a finished `Protocol`
    pub(crate) fn build(self) -> Protocol {
        let mut protocol = Protocol {
            unique_id: self.protocol_id,

            channel_types: BTreeMap::new(),
            channels: Vec::with_capacity(self.channels.len()),
        };

        let mut idx: ChannelId = 0;
        for (ctype, config) in &self.channels {
            protocol.channel_types.insert(*ctype, idx);
            protocol.channels.push((*ctype, config.clone()));
            idx = idx.checked_add(1).expect("Channel limit exceeded!");
        }

        protocol
    }
}

pub trait ProtocolAppExts {
    fn gen_protocol_id<H: std::hash::Hash>(&mut self, val: H);
    fn add_net_channel<T: Channel>(&mut self, config: ChannelConfig);
}

impl ProtocolAppExts for App {
    fn gen_protocol_id<H: std::hash::Hash>(&mut self, val: H) {
        // SipHasher is used directly to prevent the generated ID changing between Rust releases,
        // as noted in the documentation for DefaultHasher
        #[allow(deprecated)]
        let mut hasher = SipHasher::default();
        hasher.write_u16(STARDUST_PROTOCOL_VERSION);
        val.hash(&mut hasher);

        let mut builder = self.world.get_resource_mut::<ProtocolBuilder>()
            .expect("StardustSharedPlugin should have been added before this");
        let hash = hasher.finish().to_be_bytes();
        let crammed = [hash[0], hash[1], hash[2], hash[3]];
        builder.protocol_id = u32::from_be_bytes(crammed);
    }

    fn add_net_channel<T: Channel>(&mut self, config: ChannelConfig) {
        let mut builder = self.world.get_resource_mut::<ProtocolBuilder>()
            .expect("StardustSharedPlugin should have been added before this");
        builder.add_channel::<T>(config);
    }
}
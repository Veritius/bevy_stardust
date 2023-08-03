use std::{marker::PhantomData, sync::{Arc, Mutex}};
use bevy::prelude::*;
use crate::shared::{octetstring::OctetString, channels::id::Channel};

/// Used to write octet strings to remote peers. No associated type or channel information, and only accessible in `TransportSendPackets` for use by transport layers.
/// 
/// This is only returned for use in transport layers. Use `OutgoingOctetStrings<T>`, accessible in Bevy systems as a resource.
pub struct OutgoingOctetStringsUntyped {
    targets: Vec<(SendTarget, usize)>,
    octets: Vec<OctetString>,
}

impl OutgoingOctetStringsUntyped {
    pub(in crate::shared) fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            targets: Default::default(),
            octets: Default::default(),
        }))
    }

    pub(in crate) fn send(&mut self, target: SendTarget, octets: impl Into<OctetString>) {
        self.octets.push(octets.into());
        let idx = self.octets.len() - 1;
        self.targets.push((target, idx));
    }

    /// Creates an iterator over octets and targets to send them to.
    pub fn read(&self) -> impl Iterator<Item = (&SendTarget, &OctetString)> {
        struct OutgoingOctetStringsReader<'a> {
            data: &'a OutgoingOctetStringsUntyped,
            index: usize
        }
        
        impl<'a> Iterator for OutgoingOctetStringsReader<'a> {
            type Item = (&'a SendTarget, &'a OctetString);

            fn next(&mut self) -> Option<Self::Item> {
                let (target, index) = self.data.targets.get(self.index)?;
                let octets = self.data.octets.get(*index)?;
                Some((target, octets))
            }
        }

        OutgoingOctetStringsReader {
            data: &self,
            index: 0,
        }
    }
}

/// Used to write octet strings to remote peers.
/// You should use the client/server variations of `ChannelReader` and `ChannelWriter`
#[derive(Resource)]
pub struct OutgoingOctetStrings<T: Channel> {
    internal: Arc<Mutex<OutgoingOctetStringsUntyped>>,
    phantom: PhantomData<T>,
}

impl<T: Channel> OutgoingOctetStrings<T> {
    pub(in crate::shared) fn new(internal: Arc<Mutex<OutgoingOctetStringsUntyped>>) -> Self {
        Self {
            internal,
            phantom: PhantomData,
        }
    }

    pub fn send(&mut self, target: SendTarget, octets: impl Into<OctetString>) {
        self.internal.lock().unwrap().send(target, octets);
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SendTarget {
    Single(Entity),
    Multiple(Box<[Entity]>),
    Broadcast,
}

impl From<Entity> for SendTarget {
    fn from(value: Entity) -> Self {
        Self::Single(value)
    }
}

impl TryFrom<&[Entity]> for SendTarget {
    type Error = ();
    fn try_from(value: &[Entity]) -> Result<Self, Self::Error> {
        match value.len() {
            0 => Err(()),
            1 => Ok(Self::Single(value[0].clone())),
            _ => Ok(Self::Multiple(value.iter().cloned().collect::<Box<[Entity]>>()))
        }
    }
}
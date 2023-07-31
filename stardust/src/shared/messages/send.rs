use std::marker::PhantomData;
use bevy::prelude::*;
use crate::shared::{octetstring::OctetString, channels::id::Channel};

#[derive(Resource, Default)]
pub struct OutgoingOctetStrings<T: Channel> {
    targets: Vec<(SendTarget, usize)>,
    octets: Vec<OctetString>,
    phantom: PhantomData<T>,
}

impl<T: Channel> OutgoingOctetStrings<T> {
    pub fn send(&mut self, target: SendTarget, octets: OctetString) {
        self.octets.push(octets);
        let idx = self.octets.len() - 1;
        self.targets.push((target, idx));
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SendTarget {
    Single(Entity),
    Multiple(Box<[Entity]>),
    Broadcast,
}
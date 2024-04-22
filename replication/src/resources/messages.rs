use std::marker::PhantomData;
use bevy::prelude::*;
use crate::prelude::*;

#[derive(Resource)]
pub(super) struct ResourceSerialisationFunctions<T: ReplicableResource> {
    pub fns: SerialisationFunctions<T>,
}

#[derive(TypePath, Default)]
pub(super) struct ResourceReplicationMessages<T: ReplicableResource>(PhantomData<T>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum MessageHeader {
    Insert,
    Remove,
}

impl TryFrom<u8> for MessageHeader {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Insert,
            1 => Self::Remove,
            _ => { return Err(()); }
        })
    }
}

impl From<MessageHeader> for u8 {
    fn from(value: MessageHeader) -> Self {
        match value {
            MessageHeader::Insert => 0,
            MessageHeader::Remove => 1,
        }
    }
}

#[test]
fn header_matching_test() {
    const VARIANTS: &[MessageHeader] = &[
        MessageHeader::Insert,
        MessageHeader::Remove,
    ];

    for initial in VARIANTS.iter().cloned() {
        let byte: u8 = initial.into();
        let back = MessageHeader::try_from(byte).unwrap();
        assert_eq!(initial, back);
    }
}
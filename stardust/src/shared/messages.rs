use std::{marker::PhantomData, collections::HashMap};
use bevy::prelude::Resource;
use crate::{channel::Channel, types::NetworkUserId};

/// A thin wrapper around a `Vec<u8>` containing the *payload* of a message. Stardust automatically assembles the header.
pub struct Message(Box<[u8]>);
impl Message {
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Message(bytes.into_boxed_slice())
    }

    /// Consumes the `Message`, returning the internal storage.
    pub fn bytes(self) -> Box<[u8]> {
        self.0
    }
}

#[derive(Resource)]
pub struct MessageReader<T: Channel> {
    messages: HashMap<NetworkUserId, Messages>,
    phantom: PhantomData<T>,
}

impl<T: Channel> MessageReader<T> {
    pub fn read_from(&mut self, user: NetworkUserId) -> Option<&Messages> {
        if let Some(v) = self.messages.get(&user) {
            Some(v)
        } else {
            None
        }
    }
}

pub struct Messages {
    index: usize,
    slice: Box<[Message]>,
}

impl Messages {
    pub fn next(&mut self) -> Option<&Message> {
        let x = self.slice.get(self.index);
        self.index += 1;
        return x;
    }
}

#[derive(Resource)]
pub struct MessageWriter<T: Channel> {
    messages: HashMap<NetworkUserId, Vec<Message>>,
    phantom: PhantomData<T>,
}

impl<T: Channel> MessageWriter<T> {
    pub fn send(&mut self, to: NetworkUserId, message: Message) {
        if let Some(val) = self.messages.get_mut(&to) {
            val.push(message);
        } else {
            self.messages.insert(to, vec![message]);
        }
    }
}
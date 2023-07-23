use std::{marker::PhantomData, collections::HashMap};
use bevy::prelude::Resource;
use crate::{channel::Channel, types::NetworkUserId};

/// A thin wrapper around a `Vec<u8>` containing the *payload* of a message. Stardust automatically assembles the header.
pub struct Message(Vec<u8>);
impl Message {
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Message(bytes)
    }

    /// Consumes the `Message`, returning the internal storage.
    pub fn bytes(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Resource)]
pub struct MessageReader<T: Channel> {
    messages: HashMap<NetworkUserId, (usize, Box<[Message]>)>,
    phantom: PhantomData<T>,
}

impl<T: Channel> MessageReader<T> {
    pub fn read_from(&mut self, user: NetworkUserId) -> Option<MessageReaderIter<T>> {
        todo!()
    }
}

pub struct MessageReaderIter<'a, T: Channel> {
    reader: &'a mut (usize, &'a [Message]),
    phantom: PhantomData<T>,
}

impl<'a, T: Channel> Iterator for MessageReaderIter<'a, T> {
    type Item = &'a Message;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
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
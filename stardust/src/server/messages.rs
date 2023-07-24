use std::{marker::PhantomData, collections::HashMap};
use bevy::prelude::{Resource, Entity};
use crate::{shared::channel::Channel, shared::bytes::OwnedByteStore};

/// A thin wrapper around a byte array containing the *payload* of a message. Stardust automatically assembles the header.
pub struct Message(Box<[u8]>);

impl Message {
    /// Returns an iterable slice of the bytes in the array.
    pub fn read(&self) -> &[u8] {
        &self.0
    }

    /// Consumes the `Message`, returning the internal storage.
    pub fn bytes(self) -> Box<[u8]> {
        self.0
    }
}

impl<T: OwnedByteStore> From<T> for Message {
    fn from(value: T) -> Self {
        Self(value.into_boxed())
    }
}

/// Stores messages for reading by systems.
#[derive(Resource)]
pub struct ChannelReader<T: Channel> {
    pub(crate) messages: HashMap<Entity, Messages>,
    phantom: PhantomData<T>,
}

impl<T: Channel> ChannelReader<T> {
    pub fn read_user(&mut self, user: Entity) -> Option<&mut Messages> {
        if let Some(v) = self.messages.get_mut(&user) {
            Some(v)
        } else {
            None
        }
    }
}

/// Stores the channel messages from specific users.
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

    /// Exposes the internal message storage, allowing reading without advancing the counter.
    #[cfg(feature="expose_internals")]
    pub fn internal_buffer(&self) -> &[Message] {
        &self.slice
    }
}

#[derive(Resource)]
pub struct ChannelWriter<T: Channel> {
    pub single: HashMap<Entity, Vec<Message>>,
    pub multiple: Vec<(Box<[Entity]>, Message)>,
    pub broadcast: Vec<Message>,
    phantom: PhantomData<T>,
}

impl<T: Channel> ChannelWriter<T> {
    /// Sends a message to one user.
    pub fn send(&mut self, user: Entity, message: impl Into<Message>) {
        if let Some(val) = self.single.get_mut(&user) {
            val.push(message.into());
        } else {
            self.single.insert(user, vec![message.into()]);
        }
    }

    /// Sends a message to several users.
    pub fn send_multi(&mut self, users: &[Entity], message: impl Into<Message>) {
        self.multiple.push((users.clone().into(), message.into()));
    }

    /// Sends a message on to all users.
    pub fn broadcast(&mut self, message: impl Into<Message>) {
        self.broadcast.push(message.into());
    }
}
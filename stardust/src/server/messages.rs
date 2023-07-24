use bevy::{ecs::system::SystemParam, prelude::ResMut};
use crate::{
    shared::{messages::{
        ChannelWriter as ChannelWriterInternal,
        ChannelReader as ChannelReaderInternal,
        Messages, Message,
    }, serialisation::{ManualBitSerialisation, DefaultBitWriter, BitWriter}, user::NetworkUserId},
    channel::Channel,
};

/// Allows simple reading of messages in a channel.
#[derive(SystemParam)]
pub struct ChannelReader<'w, T: Channel> {
    internal: ResMut<'w, ChannelReaderInternal<T>>,
}

impl<'w, T: Channel> ChannelReader<'w, T> {
    /// Returns an object that can be used to read the messages of a specific user.
    pub fn read_user(&mut self, user: NetworkUserId) -> Option<&mut Messages> {
        self.internal.read_user(user)
    }
}

/// Allows sending messages to a network channel.
#[derive(SystemParam)]
pub struct ChannelWriter<'w, T: Channel> {
    internal: ResMut<'w, ChannelWriterInternal<T>>,
}

impl<'w, T: Channel> ChannelWriter<'w, T> {
    /// Sends a message to one user.
    pub fn send(&mut self, user: NetworkUserId, message: impl Into<Message>) {
        let message: Message = message.into();
        self.internal.send(user, message);
    }

    /// Serialises and sends a message to one user.
    pub fn send_ser<M: ManualBitSerialisation>(&mut self, user: NetworkUserId, message: M) {
        let mut writer = DefaultBitWriter::new();
        message.serialise(&mut writer);
        self.internal.send(user, writer.to_bytes());
    }

    /// Sends a message to several users.
    pub fn send_multi(&mut self, users: &[NetworkUserId], message: impl Into<Message>) {
        self.internal.send_multi(users, message);
    }

    /// Serialises and sends a message to several users.
    pub fn send_multi_ser<M: ManualBitSerialisation>(&mut self, users: &[NetworkUserId], message: M) {
        let mut writer = DefaultBitWriter::new();
        message.serialise(&mut writer);
        self.send_multi(users, writer.to_bytes());
    }

    /// Sends a message to all users.
    pub fn broadcast(&mut self, message: impl Into<Message>) {
        let message: Message = message.into();
        self.internal.broadcast(message);
    }

    /// Serialises and sends a message to all users.
    pub fn broadcast_ser<M: ManualBitSerialisation>(&mut self, message: M) {
        let mut writer = DefaultBitWriter::new();
        message.serialise(&mut writer);
        self.internal.broadcast(writer.to_bytes());
    }
}
mod datagrams;
mod receiving;
mod sending;
mod streams;

use std::ops::{Deref, DerefMut};
use bevy::prelude::*;
use quiche::ConnectionId;
use crate::plugin::QuicSystems;

pub(crate) fn setup(app: &mut App) {
    app.add_systems(PreUpdate, receiving::endpoints_receive_datagrams_system
        .in_set(QuicSystems::ReceivePackets));

    app.add_systems(PreUpdate, sending::endpoints_transmit_datagrams_system
        .in_set(QuicSystems::TransmitPackets));
}

pub(crate) struct QuicheConnection {
    inner: quiche::Connection,
}

impl QuicheConnection {
    pub fn new(value: quiche::Connection) -> Self {
        Self {
            inner: value,
        }
    }
}

impl Deref for QuicheConnection {
    type Target = quiche::Connection;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for QuicheConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

fn issue_connection_id() -> ConnectionId<'static> {
    ConnectionId::from_vec(rand::random::<[u8; 16]>().into())
}
// mod connect;
mod connections;
// mod datagrams;
mod endpoints;
// mod receiving;
// mod sending;
// mod streams;

// use std::ops::{Deref, DerefMut};
use bevy::prelude::*;
// use quiche::ConnectionId;
use crate::{backend::QuicBackend, plugin::QuicSystems};

// pub(crate) use endpoints::{
//     build_client,
//     build_server,
//     build_dual,
// };

// pub(crate) fn setup(app: &mut App) {
//     app.add_systems(PreUpdate, receiving::endpoints_receive_datagrams_system
//         .in_set(QuicSystems::ReceivePackets));

//     app.add_systems(PostUpdate, connect::connection_attempt_events_system
//         .before(sending::endpoints_transmit_datagrams_system));

//     app.add_systems(PostUpdate, sending::endpoints_transmit_datagrams_system
//         .in_set(QuicSystems::TransmitPackets));
// }

// pub(crate) struct QuicheConnection {
//     inner: quiche::Connection,

//     out_sid_idx: u64,
// }

// impl QuicheConnection {
//     pub fn new(value: quiche::Connection) -> Self {
//         Self {
//             inner: value,

//             out_sid_idx: 0,
//         }
//     }
// }

// impl Deref for QuicheConnection {
//     type Target = quiche::Connection;
    
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl DerefMut for QuicheConnection {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

// fn issue_connection_id() -> ConnectionId<'static> {
//     ConnectionId::from_vec(rand::random::<[u8; 16]>().into())
// }

/// Uses the `quiche` crate as a backend QUIC implementation.
/// 
/// Only enabled with the `quiche` feature flag.
#[derive(TypePath)]
pub struct Quiche {
    _hidden: (),
}

impl QuicBackend for Quiche {
    type EndpointState = endpoints::QuicheEndpoint;
    type ConnectionState = connections::QuicheConnection;
}

impl Quiche {
    /// Creates a new [`Quiche`] backend instance.
    pub fn new() -> Self {
        Self { _hidden: () }
    }
}
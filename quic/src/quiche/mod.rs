mod connect;
mod datagrams;
mod receiving;
mod sending;
mod streams;

use std::ops::{Deref, DerefMut};
use anyhow::Result;
use bevy::prelude::*;
use quiche::{Config, ConnectionId};
use crate::{plugin::QuicSystems, Credentials, TrustAnchors};

pub(crate) fn setup(app: &mut App) {
    app.add_systems(PreUpdate, receiving::endpoints_receive_datagrams_system
        .in_set(QuicSystems::ReceivePackets));

    app.add_systems(PostUpdate, connect::connection_attempt_events_system
        .before(sending::endpoints_transmit_datagrams_system));

    app.add_systems(PostUpdate, sending::endpoints_transmit_datagrams_system
        .in_set(QuicSystems::TransmitPackets));
}

pub(crate) fn quiche_config(
    trust_anchors: Option<TrustAnchors>,
    credentials: Option<Credentials>,
) -> Result<Config> {
    use boring::ssl::{SslContextBuilder, SslMethod};

    let mut tls = SslContextBuilder::new(SslMethod::tls())?;

    // Add the trust anchors
    if let Some(trust_anchors) = trust_anchors {
        tls.set_cert_store(trust_anchors.into_boring_x509_store());
    }

    // Add credentials
    if let Some(credentials) = credentials {
        // Private key
        tls.set_private_key(credentials.private_key.as_boring_pkey_ref())?;

        // First certificate in chain
        let mut iter = credentials.certificates.iter();
        tls.set_certificate(iter.next().unwrap().as_boring_x509_ref())?;

        // The rest of the chain
        for cert in iter {
            tls.add_extra_chain_cert(cert.as_boring_x509())?;
        }
    }

    let mut config = Config::with_boring_ssl_ctx_builder(quiche::PROTOCOL_VERSION, tls)?;
    config.enable_dgram(true, todo!(), todo!());

    todo!()
}

pub(crate) struct QuicheConnection {
    inner: quiche::Connection,

    out_sid_idx: u64,
}

impl QuicheConnection {
    pub fn new(value: quiche::Connection) -> Self {
        Self {
            inner: value,

            out_sid_idx: 0,
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
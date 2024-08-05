use std::net::SocketAddr;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bevy_stardust_quic::SendContext;
use quiche::RecvInfo;
use crate::events::{ConnectionEvent, ConnectionEvents};

/// A QUIC connection.
#[derive(Component, Reflect)]
#[reflect(from_reflect = false, Component)]
pub struct Connection {
    #[reflect(ignore)]
    inner: Box<ConnectionInner>,
}

impl Connection {
    pub(crate) fn new(
        address: SocketAddr,
        endpoint: SocketAddr,
        quiche: quiche::Connection,
        events: ConnectionEvents,
    ) -> Self {
        Self {
            inner: Box::new(ConnectionInner {
                address,
                endpoint,
                
                quiche,
                state: bevy_stardust_quic::Connection::new(),

                events,
            })
        }
    }
}

struct ConnectionInner {
    address: SocketAddr,
    endpoint: SocketAddr,

    quiche: quiche::Connection,
    state: bevy_stardust_quic::Connection,

    events: ConnectionEvents,
}

pub(crate) fn connection_event_handling_system(
    mut connections: Query<&mut Connection>,
) {
    connections.par_iter_mut().for_each(|mut connection| {
        let recv_info = RecvInfo {
            from: connection.inner.endpoint,
            to: connection.inner.address,
        };

        loop {
            // Try to read an event
            let event = match connection.inner.events.try_recv() {
                Ok(Some(event)) => event,
                Ok(None) => break,
                Err(_) => todo!(),
            };

            match event {
                ConnectionEvent::RecvPacket { mut payload } => {
                    // Packet was received
                    if let Err(err) = connection.inner.quiche.recv(
                        &mut payload,
                        recv_info,
                    ) {
                        todo!()
                    }
                },

                ConnectionEvent::Closed => todo!(),
            }
        }

        while let Some(event) = connection.inner.state.poll() {
            match event {
                bevy_stardust_quic::ConnectionEvent::Overheated => todo!(),

                bevy_stardust_quic::ConnectionEvent::StreamEvent(event) => {
                    match event {
                        bevy_stardust_quic::StreamEvent::Open { id } => {
                            /* Do nothing */
                        },

                        bevy_stardust_quic::StreamEvent::Transmit { id, chunk } => {
                            todo!()
                        },

                        bevy_stardust_quic::StreamEvent::SetPriority { id, priority } => {
                            todo!()
                        },

                        bevy_stardust_quic::StreamEvent::Reset { id } => {
                            todo!()
                        },

                        bevy_stardust_quic::StreamEvent::Finish { id } => {
                            todo!()
                        },

                        bevy_stardust_quic::StreamEvent::Stop { id } => {
                            todo!()
                        },
                    }
                },

                bevy_stardust_quic::ConnectionEvent::TransmitDatagram(dgram) => {
                    if let Err(err) = connection.inner.quiche.dgram_send(&dgram) {
                        todo!()
                    }
                },
            }
        }
    });
}

pub(crate) fn connection_message_recv_system(
    mut connections: Query<(&mut Connection, &mut PeerMessages<Incoming>)>,
) {
    connections.par_iter_mut().for_each(|(mut connection, mut messages)| {
        while let Some(message) = connection.inner.state.poll_messages() {
            messages.push_one(message);
        }
    });
}

pub(crate) fn connection_message_send_system(
    channels: Channels,
    mut connections: Query<(&mut Connection, &PeerMessages<Outgoing>)>,
) {
    connections.par_iter_mut().for_each(|(mut connection, outgoing)| {
        // Create the send context device
        let send_context = SendContext {
            registry: channels.as_ref(),
            dgram_max_size: connection.inner.quiche.dgram_max_writable_len().unwrap_or(1280),
        };

        // Handle the outgoing message queue
        connection.inner.state.handle_outgoing(send_context, outgoing);
    });
}
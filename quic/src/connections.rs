use std::{collections::BTreeMap, time::Instant};
use anyhow::{bail, Result};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bytes::Bytes;
use endpoints::perform_transmit;
use quinn_proto::{coding::Codec, Connection, ConnectionHandle, Dir, FinishError, StreamId, VarInt};
use streams::{StreamFrameHeader, StreamOpenHeader};
use crate::*;

/// A QUIC connection, attached to an endpoint.
/// 
/// # Safety
/// This component must always stay in the same [`World`] as it was created in.
/// Being put into another `World` will lead to undefined behavior.
#[derive(Component)]
pub struct QuicConnection {
    pub(crate) handle: ConnectionHandle,
    pub(crate) inner: Box<Connection>,

    channel_streams: BTreeMap<ChannelId, StreamId>,
}

impl QuicConnection {
    pub(crate) fn new(
        handle: ConnectionHandle,
        inner: Box<Connection>,
    ) -> Self {
        Self {
            handle,
            inner,

            channel_streams: BTreeMap::new(),
        }
    }

    /// Begins closing the connection.
    pub fn close(&mut self, reason: Bytes) {
        self.inner.close(
            Instant::now(),
            DisconnectCode::AppDisconnect.try_into().unwrap(),
            reason
        );
    }

    /// Closes a stream used to send Stardust messages, releasing some resources.
    /// This is useful as an optimisation for channels that are never used after a certain point.
    /// If the channel continues to be used, a new stream will be opened.
    pub fn close_stardust_stream(&mut self, channel: ChannelId) -> Result<()> {
        if let Some(stream_id) = self.channel_streams.get(&channel) {
            // Try to close the stream
            match self.inner.send_stream(*stream_id).finish() {
                Ok(()) => {},
                Err(FinishError::ClosedStream) => {},
                Err(FinishError::Stopped(code)) => bail!("Stream was stopped by remote: {code}"),
            }

            // Remove from map
            self.channel_streams.remove(&channel);
            return Ok(())
        } else {
            // No work to do
            return Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DisconnectCode {
    Invalid,

    Unspecified,
    AppDisconnect,
    NotListening,
}

impl From<VarInt> for DisconnectCode {
    fn from(value: VarInt) -> Self {
        use DisconnectCode::*;
        match u64::from(value) {
            0 => Unspecified,
            1 => AppDisconnect,
            2 => NotListening,

            _ => Invalid,
        }
    }
}

impl TryFrom<DisconnectCode> for VarInt {
    type Error = ();

    fn try_from(value: DisconnectCode) -> Result<Self, Self::Error> {
        use DisconnectCode::*;
        return Ok(VarInt::from_u32(match value {
            // Special case: this variant can't be sent
            Invalid => { return Err(()) },

            Unspecified => 0,
            AppDisconnect => 1,
            NotListening => 2,
        }));
    }
}

pub(crate) fn connection_message_sender_system(
    registry: Res<ChannelRegistry>,
    mut connections: Query<(Entity, &mut QuicConnection, &NetworkMessages<Outgoing>)>,
) {
    // Iterate all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, outgoing)| {
        // Logging stuff
        let trace_span = trace_span!("Sending packets from endpoint", endpoint=?entity);
        let _entered = trace_span.entered();

        // Tiny scratch space for some operations
        let mut scr = Vec::with_capacity(4);

        // Iterate over all channels
        for (channel, messages) in outgoing.iter() {
            // Get the channel data
            let channel_data = match registry.channel_config(channel) {
                Some(channel_data) => channel_data,
                None => {
                    error!("Tried to send a message to a channel that didn't exist ({channel:?})");
                    continue;
                },
            };

            // Different channels have different config requirements
            match (channel_data.reliable, channel_data.ordered) {
                (ReliabilityGuarantee::Unreliable, _) => todo!(),

                // Reliable transport is reliable
                // This forces us to use streams
                (ReliabilityGuarantee::Reliable, _) => {
                    // Get the stream ID of this channel
                    let stream_id: StreamId = match connection.channel_streams.get(&channel) {
                        Some(v) => *v,
                        None => {
                            // Wasn't in the map, we have to open a new stream
                            let stream_id = connection.inner.streams().open(Dir::Uni).unwrap();

                            // Add the StreamOpenHeader to the stream
                            // This informs the remote connection of what this stream is for
                            StreamOpenHeader::StardustReliable { channel }.encode(&mut scr);
                            connection.inner.send_stream(stream_id).write(&scr).unwrap();

                            // Add the stream to the map
                            connection.channel_streams.insert(channel, stream_id);
                            stream_id
                        },
                    };

                    // Iterate over all messages and send them
                    let mut send_stream = connection.inner.send_stream(stream_id);
                    for payload in messages.iter().cloned() {
                        // Stream frame header stuff
                        scr.clear(); // Clear the scratch space
                        StreamFrameHeader { length: payload.len() }.encode(&mut scr);

                        // TODO: Handle error cases
                        send_stream.write(&scr[..]).unwrap();
                        send_stream.write(&payload[..]).unwrap();
                    }
                },
            }
        }
    });
}

pub(crate) fn connection_datagram_send_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    connections: Query<&mut QuicConnection>,
) {
    endpoints.par_iter_mut().for_each(|(entity, mut endpoint)| {
        // Logging stuff
        let trace_span = trace_span!("Sending packets from endpoint", endpoint=?entity);
        let _entered = trace_span.entered();

        // Some stuff related to the endpoint
        let endpoint = endpoint.as_mut();
        let socket = &mut endpoint.socket;

        // Allocate a buffer to store messages in
        let mut buf = Vec::with_capacity(2048); // TODO: Make this based on MTU

        // Iterate over all connections associated with this endpoint
        let entities = endpoint.entities.iter();
        for (handle, entity) in entities {
            // Logging stuff
            let trace_span = trace_span!("Polling connection", connection=?entity, handle=?handle);
            let _entered = trace_span.entered();

            // SAFETY: Endpoints will only access the connections they have created
            let query_item = unsafe { connections.get_unchecked(*entity) };
            let mut connection = match query_item {
                Ok(connection) => connection,
                Err(err) => todo!(),
            };

            // Handle timeouts
            connection.inner.handle_timeout(Instant::now());

            // Repeatedly poll transmit until the connection no longer wants to send any more packets
            let mut send_count: u32 = 0;
            while let Some(transmit) = connection.inner.poll_transmit(Instant::now(), 10, &mut buf) {
                perform_transmit(socket, &buf, transmit);
                send_count += 1;
            }

            // Record the amount of packets we've sent
            _entered.record("sends", send_count);
        }
    });
}
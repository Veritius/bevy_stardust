use bevy::log::trace;
use bevy_stardust::{messages::{ChannelId, MessageIter}, prelude::*};
use quinn_proto::Dir;
use crate::{datagrams::*, streams::*, QuicConnection};

pub(super) fn stardust_transmit_many(
    config: &ChannelConfiguration,
    connection: &mut QuicConnection,
    queued_datagrams: &mut Vec<(u32, Datagram)>,
    channel: ChannelId,
    messages: MessageIter,
) {
    use ChannelConsistency::*;
    match config.consistency {
        UnreliableUnordered => {
            queued_datagrams.extend(messages.map(|m| (config.priority, Datagram {
                header: DatagramHeader {
                    purpose: DatagramPurpose::StardustUnordered {
                        channel: channel.into(),
                    }
                },

                payload: m.into(),
            })));
        },

        UnreliableSequenced => {
            let sequencer = connection.sequencers
                .entry(channel)
                .or_insert_with(|| DatagramSequencer::new());

            queued_datagrams.extend(messages.map(|m| (config.priority, Datagram {
                header: DatagramHeader {
                    purpose: DatagramPurpose::StardustSequenced {
                        channel: channel.into(),
                        sequence: sequencer.next(),
                    }
                },

                payload: m.into(),
            })));
        },

        ReliableUnordered => {
            for message in messages {
                // Open a new outgoing, unidirectional stream
                let id = connection.inner.streams().open(Dir::Uni).unwrap();
                let mut stream = connection.inner.send_stream(id);
                stream.set_priority(map_stardust_priority(config.priority)).unwrap();
                trace!(?channel, stream=?id, "Opened stream for reliable unordered messages");

                // Create a new sender
                let mut send = Send::new(SendInit::StardustTransient { channel: channel.into() });

                // Add the message
                send.push(message.into());

                // Try to write as much as possible to the stream
                match send.write(&mut stream) {
                    // The entire send buffer was written
                    StreamWriteOutcome::Complete => {
                        trace!(?channel, stream=?id, "ReliableUnordered stream did full transmit and was finished");
                        stream.finish().unwrap();
                    },

                    // Only a portion of the send buffer was written
                    StreamWriteOutcome::Partial(_) |
                    StreamWriteOutcome::Blocked => {
                        let boxed = Box::new(send);
                        connection.senders.insert(id, boxed);
                    },

                    StreamWriteOutcome::Error(err) => {
                        trace!(stream=?id, "Stream send failed: {err:?}");
                        continue;
                    },
                }
            }
        },

        ReliableOrdered => {
            // Get the ID of the channel
            let id = connection.channels.entry(channel).or_insert_with(|| {
                // Open a new outgoing, unidirectional stream
                let id = connection.inner.streams().open(Dir::Uni).unwrap();
                connection.inner.send_stream(id).set_priority(map_stardust_priority(config.priority)).unwrap();
                trace!(?channel, stream=?id, "Opened stream for reliable ordered messages");
                id
            }).clone();

            // Get the sender queue
            let send = connection.senders.entry(id).or_insert_with(|| {
                Box::new(Send::new(SendInit::StardustPersistent { channel: channel.into() }))
            }).as_mut();

            // Put all messages into the sender
            for message in messages {
                send.push(message.into());
            }

            // Try to write as much as possible to the stream
            let mut stream = connection.inner.send_stream(id);
            match send.write(&mut stream) {
                StreamWriteOutcome::Error(err) => {
                    trace!(stream=?id, "Stream send failed: {err:?}");
                    return;
                },

                _ => {},
            }
        },

        _ => unimplemented!()
    }
}

#[inline]
fn map_stardust_priority(priority: u32) -> i32 {
    TryInto::<i32>::try_into(priority).unwrap_or(i32::MAX)
}
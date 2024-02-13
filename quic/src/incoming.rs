use std::{io::ErrorKind, time::Instant};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bytes::BytesMut;
use untrusted::{EndOfInput, Input, Reader};
use crate::{QuicConnection, QuicEndpoint};

pub(super) fn quic_process_incoming_system(
    mut endpoints: Query<&mut QuicEndpoint>,
    mut connections: Query<&mut QuicConnection>,
    channels: Res<ChannelRegistry>,
    mut writer: NetworkIncomingWriter,
) {
    // Receive as many packets as we can
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        let mut scratch = Vec::with_capacity(1472); // todo make this configurable

        loop {
            match endpoint.udp_socket.recv_from(&mut scratch) {
                // Packet received, forward it to the endpoint
                Ok((bytes, address)) => {
                    endpoint.inner.get_mut().handle(
                        Instant::now(),
                        address,
                        None,
                        None,
                        BytesMut::from(&scratch[..bytes]),
                    );
                },

                // We've run out of packets to read
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    break
                },

                // Actual IO error
                Err(e) => {
                    error!("IO error while reading packets: {e}");
                    break
                }
            }
        }
    });

    // Process packets and turn them into usable messages
    let cid_len = crate::misc::bytes_for_channel_id(channels.channel_count());
    // let packets = Mutex::new(Vec::new());
    connections.par_iter_mut().for_each(|mut connection| {
        // let mut local_packets = Vec::new();
        let mut connection = connection.inner.get_mut();

        // Read datagrams
        let mut datagrams = connection.datagrams();
        while let Some(datagram) = datagrams.recv() {
            let mut reader = Reader::new(Input::from(&datagram));
            match read_datagram(&mut reader, &channels, cid_len) {
                Ok((channel_id, ordering, payload)) => {
                    todo!()
                },
                Err(_) => continue,
            }
        }
    });
}

fn read_datagram(
    reader: &mut Reader<'_>,
    channels: &ChannelRegistry,
    cid_len: u8
) -> Result<(ChannelId, u16, Bytes), EndOfInput> {
    // Channel ID
    let cid_bytes = reader.read_bytes(cid_len as usize)?.as_slice_less_safe();
    let cid = ChannelId::from(match cid_len {
        1 => { u32::from_be_bytes([cid_bytes[0], 0, 0, 0]) },
        2 => { u32::from_be_bytes([cid_bytes[0], cid_bytes[1], 0, 0]) }
        3 => { u32::from_be_bytes([cid_bytes[0], cid_bytes[1], cid_bytes[2], 0]) }
        4 => { u32::from_be_bytes(cid_bytes.try_into().unwrap()) }
        0 => panic!(), // Handle this case somehow
        _ => panic!(), // This shouldn't happen
    });

    // Ordering number
    let ordering = {
        let data = channels.get_from_id(cid);
        if data.is_none() { return Err(EndOfInput); } // Make a custom error type for this
        if data.unwrap().ordered != OrderingGuarantee::Unordered {
            let ordering = reader.read_bytes(2)?.as_slice_less_safe();
            u16::from_be_bytes(ordering.try_into().unwrap())
        } else {
            0 // The value returned here is irrelevant since the channel isn't ordered anyway
        }
    };

    // Message payload
    let payload = reader.read_bytes_to_end().as_slice_less_safe();
    let payload = Bytes::from(payload.to_owned());

    Ok((cid, ordering, payload))
}
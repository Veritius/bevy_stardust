use std::{io::ErrorKind, time::Instant};
use bevy::prelude::*;
use bytes::BytesMut;
use crate::{Connection, Endpoint};

pub(crate) fn udp_recv_system(
    mut endpoints: Query<&mut Endpoint>,
    mut connections: Query<&mut Connection>,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        // Buffer for I/O operations
        debug_assert!(endpoint.recv_buf_size < 1280, "Receive buffer was too small");
        let mut buf = vec![0u8; endpoint.recv_buf_size as usize];

        // Store some things ahead of time
        let local_ip = endpoint.local_addr().ip();

        loop {
            match endpoint.socket.recv_from(&mut buf) {
                Ok((length, address)) => {
                    // More logging stuff
                    let trace_span = trace_span!("Received datagram", length, address=?address);
                    let _entered = trace_span.entered();

                    let slice = &buf[..length];

                    match endpoint.quinn.handle(
                        Instant::now(),
                        address,
                        Some(local_ip),
                        None, // TODO
                        BytesMut::from(slice),
                        &mut buf,
                    ) {
                        Some(_) => todo!(),
                        None => todo!(),
                    }
                },

                // If this occurs, it means there are no more packets to read
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,

                Err(e) => todo!(),
            }
        }
    });
}
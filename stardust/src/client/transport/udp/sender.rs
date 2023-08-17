use bevy::prelude::*;
use crate::shared::channels::outgoing::{OutgoingOctetStringsAccessor, SendTarget};
use super::RemoteServerUdpSocket;

pub(super) fn send_packets_system(
    remote: Option<Res<RemoteServerUdpSocket>>,
    outgoing: OutgoingOctetStringsAccessor,
) { 
    let Some(remote) = remote else { return };

    let iter = outgoing.by_channel();

    for outgoing in iter {
        let id = outgoing.id();
        let outgoing = outgoing.strings();

        for (target, octets) in outgoing.read() {
            // Panics if incorrect sendtargets are used.
            // Largely redundant.
            match target {
                SendTarget::Single(_) => {},
                SendTarget::Multiple(_) => unimplemented!(),
                SendTarget::Broadcast => unimplemented!(),
            }

            // TODO: Figure out a better way to do this
            let mut payload = Vec::with_capacity(3 + octets.len());
            for b in id.as_bytes() { payload.push(b); }
            for b in octets.as_slice() { payload.push(*b); }

            // Send data
            remote.0.send(&payload).unwrap();
        }
    }
}
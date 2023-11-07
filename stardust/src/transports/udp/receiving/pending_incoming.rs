use crate::transports::udp::{connections::{PendingIncoming, PendingIncomingState, Disconnected}, reliability::Reliability, COMPAT_GOOD_VERSIONS};

pub(super) fn process_pending_incoming(
    message: &[u8],
    incoming: &mut PendingIncoming,
    reliability: &mut Reliability,
    protocol: u64,
) {
    incoming.state = match incoming.state {
        PendingIncomingState::JustRegistered => read_initial_packet(message, protocol),
        PendingIncomingState::Accepted => todo!(),
        PendingIncomingState::Rejected(_) => todo!(),
    }
}

fn read_initial_packet(
    message: &[u8],
    protocol: u64,
) -> PendingIncomingState {
    // Try to parse into a string slice
    let string = match std::str::from_utf8(message) {
        Ok(v) => v,
        Err(_) => {
            return PendingIncomingState::Rejected(Disconnected::InvalidPacket)
        },
    };

    // Try to parse into a json document
    let json = match json::parse(string) {
        Ok(v) => v,
        Err(_) => {
            return PendingIncomingState::Rejected(Disconnected::InvalidPacket);
        },
    };

    // Get request type
    match json["req"].as_str() {
        // Only the req_join case exists right now
        Some("req_join") => {},
        None => {
            return PendingIncomingState::Rejected(Disconnected::MissingData);
        },
        _ => {
            return PendingIncomingState::Rejected(Disconnected::InvalidPacket)
        }
    }

    // Check transport version
    match json["transport"].as_str() {
        Some(v) => {
            let v = match v.parse::<u32>() {
                Ok(v) => v,
                Err(_) => {
                    return PendingIncomingState::Rejected(Disconnected::InvalidData)
                },
            };
            if !COMPAT_GOOD_VERSIONS.contains(&v) {
                return PendingIncomingState::Rejected(Disconnected::WrongVersion { version: v })
            }
        },
        None => {
            return PendingIncomingState::Rejected(Disconnected::InvalidData)
        }
    }

    // Check protocol hash
    match json["protocol"].as_str() {
        Some(v) => {
            match u64::from_str_radix(v, 16) {
                Ok(v) => {
                    if v != protocol {
                        return PendingIncomingState::Rejected(Disconnected::WrongProtocol { protocol: v })
                    }
                },
                Err(_) => {
                    return PendingIncomingState::Rejected(Disconnected::InvalidData)
                },
            }
        },
        None => {
            return PendingIncomingState::Rejected(Disconnected::InvalidData)
        },
    }

    // They've succeeded :)
    return PendingIncomingState::Accepted
}
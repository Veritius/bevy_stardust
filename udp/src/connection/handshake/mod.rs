mod codes;
mod packets;

use std::time::{Duration, Instant};

pub(in crate::connection) use codes::HandshakeResponseCode;

use bytes::{BufMut, Bytes};
use unbytes::Reader;
use crate::{appdata::{NetworkVersionData, PeerVersionMismatch, BANNED_MINOR_VERSIONS, TRANSPORT_VERSION_DATA}, plugin::PluginConfiguration};
use self::packets::*;
use super::{reliability::AckMemory, shared::*, ConnectionDirection};

const RESEND_DELAY: Duration = Duration::from_secs(2);

pub(super) struct HandshakeStateMachine {
    state: HandshakeStateInner,
    last_sent: Option<Instant>,
}

impl HandshakeStateMachine {
    pub fn new(direction: ConnectionDirection) -> Self {
        Self {
            state: match direction {
                ConnectionDirection::Client => HandshakeStateInner::InitiatorHello,
                ConnectionDirection::Server => HandshakeStateInner::ListenerHello,
            },
            last_sent: None,
        }
    }

    pub fn recv(
        &mut self,
        packet: Bytes,
        config: &PluginConfiguration,
        shared: &mut ConnectionShared,
    ) -> Option<HandshakeOutcome> {
        macro_rules! finish_handshake {
            (preret) => {
                #[cfg(debug_assertions)]
                { self.state = HandshakeStateInner::Finished; }
            };

            (fail, $reason:expr) => {
                finish_handshake!(preret);
                return Some(HandshakeOutcome::FailedHandshake { reason: $reason });
            };

            (pass) => {
                finish_handshake!(preret);
                return Some(HandshakeOutcome::FinishedHandshake);
            };

            ($ret:expr) => {
                finish_handshake!(preret);
                return Some($ret);
            };
        }

        // Create reader object and read the header quickly
        let mut reader = Reader::new(packet);
        if reader.remaining() < 4 { return None; }
        let header = match HandshakePacketHeader::read(&mut reader) {
            Ok(header) => header,
            Err(_) => { return None; },
        };

        // Ignore packets that are older than a certain amount
        // This is because they were probably delayed and resent
        if header.seq_ident <= shared.reliability.remote_sequence.0 {
            return None;
        }

        // If we're not in the ListenerResponse state, this packet
        // will have a response code. How we handle the response code
        // doesn't really change based on state, so we do it first here.
        if self.state != HandshakeStateInner::ListenerHello {
            // We know that we have at least enough data to read to safely let us
            // unwrap here, because we checked earlier when creating the reader.
            let code: HandshakeResponseCode = reader.read_u16().unwrap().into();

            match code {
                HandshakeResponseCode::Continue => {},
                HandshakeResponseCode::Unknown => {
                    finish_handshake!(fail, HandshakeFailureReason::WeRejected {
                        code: HandshakeResponseCode::MalformedPacket
                    });
                },
                _ => {
                    finish_handshake!(fail, HandshakeFailureReason::TheyRejected { code });
                },
            }
        }

        match self.state {
            HandshakeStateInner::InitiatorHello => {
                let packet = match ListenerHelloPacket::read(&mut reader) {
                    Err(_) => { return None },
                    Ok(packet) => packet,
                };

                // Update reliability state
                shared.reliability.ack_seq(header.seq_ident.into());
                let _ = shared.reliability.rec_ack(
                    packet.acks.ack_ident.into(),
                    AckMemory::from_u16(packet.acks.ack_memory),
                    2
                );

                // Check their version codes
                if let Err(reason) = check_version_codes(
                    &packet.tr_ver,
                    &packet.app_ver,
                    &TRANSPORT_VERSION_DATA,
                    &config.application_version.as_nvd(),
                    BANNED_MINOR_VERSIONS,
                    config.application_version.banlist,
                ) {
                    finish_handshake!(fail, reason);
                }
            },

            HandshakeStateInner::ListenerHello => {
                let packet = match InitiatorHelloPacket::read(&mut reader) {
                    Err(_) => { return None },
                    Ok(packet) => packet,
                };

                // Update reliability state
                shared.reliability.ack_seq(header.seq_ident.into());

                // Check their version codes
                if let Err(reason) = check_version_codes(
                    &packet.tr_ver,
                    &packet.app_ver,
                    &TRANSPORT_VERSION_DATA,
                    &config.application_version.as_nvd(),
                    BANNED_MINOR_VERSIONS,
                    config.application_version.banlist,
                ) {
                    finish_handshake!(fail, reason);
                }
            },

            #[cfg(debug_assertions)]
            HandshakeStateInner::Finished => unreachable!(),
        }

        // Send a packet in response
        shared.reliability.advance();
        let len = match self.state {
            HandshakeStateInner::InitiatorHello => 8,
            HandshakeStateInner::ListenerHello => 40,

            #[cfg(debug_assertions)]
            HandshakeStateInner::Finished => todo!(),
        };

        let mut buf = Vec::with_capacity(len);

        // Packets are always headed by this
        HandshakePacketHeader {
            seq_ident: shared.reliability.local_sequence.into(),
        }.write(&mut buf);
        buf.put_u16(HandshakeResponseCode::Continue as u16);

        // The rest of the packet is dependent on state
        match self.state {
            HandshakeStateInner::InitiatorHello => {
                InitiatorResponsePacket {
                    acks: HandshakePacketAcks {
                        ack_ident: shared.reliability.remote_sequence.into(),
                        ack_memory: shared.reliability.ack_memory.into_u16(),
                    },
                }.write(&mut buf);
            },

            HandshakeStateInner::ListenerHello => {
                ListenerHelloPacket {
                    tr_ver: TRANSPORT_VERSION_DATA.clone(),
                    app_ver: config.application_version.as_nvd(),
                    acks: HandshakePacketAcks {
                        ack_ident: shared.reliability.remote_sequence.into(),
                        ack_memory: shared.reliability.ack_memory.into_u16(),
                    },
                }.write(&mut buf);
            },

            #[cfg(debug_assertions)]
            HandshakeStateInner::Finished => unreachable!(),
        }

        assert_eq!(len, buf.len());

        finish_handshake!(pass);
    }

    pub fn send(
        &mut self,
        config: &PluginConfiguration,
        shared: &mut ConnectionShared,
    ) -> Option<Bytes> {
        // Check if we need to send a message at all
        if self.last_sent.is_some_and(|v| v.elapsed() < RESEND_DELAY) {
            return None;
        }

        let len = match self.state {
            HandshakeStateInner::InitiatorHello => 36,
            HandshakeStateInner::ListenerHello => 40,

            #[cfg(debug_assertions)]
            HandshakeStateInner::Finished => todo!(),
        };

        let mut buf = Vec::with_capacity(len);
        
        // Packets are always headed by this
        HandshakePacketHeader {
            seq_ident: shared.reliability.local_sequence.into(),
        }.write(&mut buf);
        buf.put_u16(HandshakeResponseCode::Continue as u16);

        match self.state {
            HandshakeStateInner::InitiatorHello => {
                InitiatorHelloPacket {
                    tr_ver: TRANSPORT_VERSION_DATA.clone(),
                    app_ver: config.application_version.as_nvd(),
                }.write(&mut buf);
            },
            HandshakeStateInner::ListenerHello => {
                ListenerHelloPacket {
                    tr_ver: TRANSPORT_VERSION_DATA.clone(),
                    app_ver: config.application_version.as_nvd(),
                    acks: HandshakePacketAcks {
                        ack_ident: shared.reliability.remote_sequence.into(),
                        ack_memory: shared.reliability.ack_memory.into_u16(),
                    }
                }.write(&mut buf);
            },

            #[cfg(debug_assertions)]
            HandshakeStateInner::Finished => unreachable!(),
        }

        debug_assert_eq!(len, buf.len());

        self.last_sent = Some(Instant::now());
        return Some(Bytes::from(buf));
    }
}

fn check_version_codes(
    peer_tr: &NetworkVersionData,
    peer_app: &NetworkVersionData,
    loc_tr: &NetworkVersionData,
    loc_app: &NetworkVersionData,
    tr_banlist: &[u32],
    app_banlist: &[u32],
) -> Result<(), HandshakeFailureReason> {
    if let Err(err) = peer_tr.check(&loc_tr, tr_banlist) {
        return Err(HandshakeFailureReason::WeRejected { code: match err {
            PeerVersionMismatch::BadIdent => HandshakeResponseCode::IncompatibleTransportIdentifier,
            PeerVersionMismatch::BadMajor => HandshakeResponseCode::IncompatibleTransportMajorVersion,
            PeerVersionMismatch::BadMinor => HandshakeResponseCode::IncompatibleTransportMinorVersion,
        }});
    }

    if let Err(err) = peer_app.check(&loc_app, app_banlist) {
        return Err(HandshakeFailureReason::WeRejected { code: match err {
            PeerVersionMismatch::BadIdent => HandshakeResponseCode::IncompatibleApplicationIdentifier,
            PeerVersionMismatch::BadMajor => HandshakeResponseCode::IncompatibleApplicationMajorVersion,
            PeerVersionMismatch::BadMinor => HandshakeResponseCode::IncompatibleApplicationMinorVersion,
        }});
    }

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HandshakeStateInner {
    InitiatorHello,
    ListenerHello,

    #[cfg(debug_assertions)]
    Finished, // for debugging
}

pub(super) enum HandshakeOutcome {
    FinishedHandshake,
    FailedHandshake {
        reason: HandshakeFailureReason,
    },
}

pub(super) enum HandshakeFailureReason {
    Timeout,
    TheyRejected { code: HandshakeResponseCode },
    WeRejected { code: HandshakeResponseCode },
}
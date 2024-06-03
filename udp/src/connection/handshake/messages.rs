use std::fmt::Debug;
use anyhow::bail;
use bytes::{BufMut, Bytes};
use unbytes::{EndOfInput, Reader};
use crate::{connection::reliability::AckMemory, sequences::SequenceId};
use super::{codes::HandshakeResponseCode, AppVersion};

static NOT_ENOUGH_SPACE: &str = "Not enough space in buffer";

pub(super) trait HandshakeMessage: Debug + Sized {
    fn recv(reader: &mut Reader) -> Result<Self, EndOfInput>;
    fn send<B: BufMut>(&self, b: &mut B) -> anyhow::Result<usize>;
}

#[derive(Debug)]
pub(super) struct InitiatorHello {
    pub tpt_ver: AppVersion,
    pub app_ver: AppVersion,
}

impl HandshakeMessage for InitiatorHello {
    fn recv(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            tpt_ver: AppVersion::from_bytes(reader)?,
            app_ver: AppVersion::from_bytes(reader)?,
        })
    }

    fn send<B: BufMut>(&self, b: &mut B) -> anyhow::Result<usize> {
        if b.remaining_mut() < 32 { bail!(NOT_ENOUGH_SPACE); }
        b.put(&self.tpt_ver.to_bytes()[..]);
        b.put(&self.app_ver.to_bytes()[..]);
        return Ok(32);
    }
}

#[derive(Debug)]
pub(super) enum ListenerHello {
    Rejected(Rejection),

    Accepted {
        tpt_ver: AppVersion,
        app_ver: AppVersion,
        ack_seq: SequenceId,
        ack_bits: AckMemory,
    },
}

impl HandshakeMessage for ListenerHello {
    fn recv(reader: &mut Reader) -> Result<Self, EndOfInput> {
        let code: HandshakeResponseCode = reader.read_u16()?.into();

        if code != HandshakeResponseCode::Continue {
            return Ok(Self::Rejected(Rejection { code, message: Bytes::new() }));
        }

        return Ok(Self::Accepted {
            tpt_ver: AppVersion::from_bytes(reader)?,
            app_ver: AppVersion::from_bytes(reader)?,
            ack_seq: SequenceId::from(reader.read_u16()?),
            ack_bits: u16_to_ack_mem(reader.read_u16()?),
        });
    }

    fn send<B: BufMut>(&self, b: &mut B) -> anyhow::Result<usize> {
        match self {
            ListenerHello::Rejected(rejection) => {
                rejection.send(b)
            },

            ListenerHello::Accepted {
                tpt_ver,
                app_ver,
                ack_seq,
                ack_bits,
            } => {
                let length = 36;
                if b.remaining_mut() < length { bail!(NOT_ENOUGH_SPACE); }

                b.put(&tpt_ver.to_bytes()[..]);
                b.put(&app_ver.to_bytes()[..]);
                b.put_u16(ack_seq.0);
                b.put_u16(ack_mem_to_u16(*ack_bits));

                return Ok(length);
            },
        }
    }
}

#[derive(Debug)]
pub(super) enum InitiatorFinish {
    Rejected(Rejection),

    Accepted {
        ack_seq: SequenceId,
        ack_bits: AckMemory,
    },
}

impl HandshakeMessage for InitiatorFinish {
    fn recv(reader: &mut Reader) -> Result<Self, EndOfInput> {
        let code: HandshakeResponseCode = reader.read_u16()?.into();

        if code != HandshakeResponseCode::Continue {
            return Ok(Self::Rejected(Rejection { code, message: Bytes::new() }));
        }

        return Ok(Self::Accepted {
            ack_seq: SequenceId::from(reader.read_u16()?),
            ack_bits: u16_to_ack_mem(reader.read_u16()?),
        });
    }

    fn send<B: BufMut>(&self, b: &mut B) -> anyhow::Result<usize> {
        match self {
            InitiatorFinish::Rejected(rejection) => {
                rejection.send(b)
            },

            InitiatorFinish::Accepted {
                ack_seq,
                ack_bits,
            } => {
                let length = 4;
                if b.remaining_mut() < length { bail!(NOT_ENOUGH_SPACE); }

                b.put_u16(ack_seq.0);
                b.put_u16(ack_mem_to_u16(*ack_bits));

                return Ok(length);
            },
        }
    }
}

fn ack_mem_to_u16(mem: AckMemory) -> u16 {
    let b = mem.into_array();
    u16::from_be_bytes([b[0], b[1]])
}

fn u16_to_ack_mem(val: u16) -> AckMemory {
    let mut b = [0u8; 16];
    let k = val.to_be_bytes();
    [b[0], b[1]] = [k[0], k[1]];
    AckMemory::from_array(b)
}

#[derive(Debug)]
pub(super) struct Rejection {
    pub code: HandshakeResponseCode,
    pub message: Bytes,
}

impl HandshakeMessage for Rejection {
    fn recv(_reader: &mut Reader) -> Result<Self, EndOfInput> {
        unimplemented!()
    }

    fn send<B: BufMut>(&self, b: &mut B) -> anyhow::Result<usize> {
        let length = 2 + self.message.len();
        if b.remaining_mut() < length { bail!(NOT_ENOUGH_SPACE); }

        b.put_u16(self.code as u16);
        b.put(&self.message[..]);

        return Ok(length);
    }
}
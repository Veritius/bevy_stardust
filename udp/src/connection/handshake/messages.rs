use std::fmt::Debug;
use anyhow::bail;
use bytes::{BufMut, Bytes};
use unbytes::{EndOfInput, Reader};
use crate::sequences::SequenceId;
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
    Rejected {
        code: HandshakeResponseCode,
        message: Bytes,
    },

    Accepted {
        tpt_ver: AppVersion,
        app_ver: AppVersion,
        ack_seq: SequenceId,
        ack_bits: u16,
    },
}

impl HandshakeMessage for ListenerHello {
    fn recv(reader: &mut Reader) -> Result<Self, EndOfInput> {
        let code: HandshakeResponseCode = reader.read_u16()?.into();

        if code != HandshakeResponseCode::Continue {
            return Ok(Self::Rejected {
                code,
                message: reader.subreader(reader.remaining()).unwrap().read_to_end(),
            });
        }

        return Ok(Self::Accepted {
            tpt_ver: AppVersion::from_bytes(reader)?,
            app_ver: AppVersion::from_bytes(reader)?,
            ack_seq: SequenceId::from(reader.read_u16()?),
            ack_bits: reader.read_u16()?,
        });
    }

    fn send<B: BufMut>(&self, b: &mut B) -> anyhow::Result<usize> {
        match self {
            ListenerHello::Rejected {
                code,
                message
            } => {
                let length = 2 + message.len();
                if b.remaining_mut() < length { bail!(NOT_ENOUGH_SPACE); }

                b.put_u16(*code as u16);
                b.put(&message[..]);

                return Ok(length);
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
                b.put_u16(*ack_bits);

                return Ok(length);
            },
        }
    }
}
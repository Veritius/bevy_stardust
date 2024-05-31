use std::fmt::Debug;
use anyhow::bail;
use bytes::BufMut;
use unbytes::{EndOfInput, Reader};
use crate::sequences::SequenceId;

use super::{codes::HandshakeResponseCode, AppVersion};

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
        if b.remaining_mut() < 32 { bail!("Not enough space in buffer"); }
        b.put(&self.tpt_ver.to_bytes()[..]);
        b.put(&self.app_ver.to_bytes()[..]);
        return Ok(32);
    }
}

pub(super) enum ListenerHello {
    Rejected {
        code: HandshakeResponseCode,
    },

    Accepted {
        tpt_ver: AppVersion,
        app_ver: AppVersion,
        ack_seq: SequenceId,
        ack_bits: u16,
    },
}
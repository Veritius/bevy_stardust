use bevy_stardust_extras::numbers::VarInt;
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub(super) struct FramedHeader {
    pub length: usize,
}

impl FramedHeader {
    pub fn read<B: Buf>(buf: &mut B) -> Result<FramedHeader, ()> {
        return Ok(Self{ 
            length: VarInt::read(buf)
                .map(|v| u64::from(v))
                .and_then(|v| usize::try_from(v).map_err(|_| ()))?,
        });
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        VarInt::try_from(self.length)
            .expect("Frame header length was too long")
            .write(buf)?;

        return Ok(());
    }

    pub fn alloc(&self) -> Result<Bytes, ()> {
        let est_len = VarInt::len_u64(self.length as u64).unwrap() as usize;
        let mut buf = BytesMut::with_capacity(est_len);
        self.write(&mut buf)?;
        return Ok(buf.freeze());
    }
}
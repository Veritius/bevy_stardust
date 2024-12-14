use bytes::{Buf, BufMut, Bytes};
use quinn_proto::{VarInt, coding::Codec};

struct Segment(Bytes);

impl Segment {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, EndOfInput> {
        let len = VarInt::decode(buf).map_err(|_| EndOfInput)?.into_inner();
        let len = usize::try_from(len).map_err(|_| EndOfInput)?;
        if buf.remaining() < len { return Err(EndOfInput) }
        return Ok(Segment(buf.copy_to_bytes(len)));
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        // Encode the length of the segment as a variable length integer
        // Unwrapping here is probably fine because I doubt anyone
        // is going to send a 46116 terabyte message...
        VarInt::from_u64(self.0.len() as u64).unwrap().encode(buf);

        // Put the rest of the segment into the buffer
        buf.put_slice(&self.0);
    }
}

struct EndOfInput;
use bevy_stardust::prelude::ChannelId;
use bevy_stardust_extras::numbers::{VarInt, Sequence};
use bytes::{Buf, BufMut, Bytes};

struct Segment(Bytes);

impl Segment {
    fn read<B: Buf>(buf: &mut B) -> Result<Self, EndOfInput> {
        let len: u64 = VarInt::read(buf).map_err(|_| EndOfInput)?.into();
        let len = usize::try_from(len).map_err(|_| EndOfInput)?;
        if buf.remaining() < len { return Err(EndOfInput) }
        return Ok(Segment(buf.copy_to_bytes(len)));
    }

    fn write<B: BufMut>(&self, buf: &mut B) -> Result<(), InsufficientSpace> {
        // Encode the length of the segment as a variable length integer
        // Unwrapping here is probably fine because I doubt anyone
        // is going to send a 46116 terabyte message...
        VarInt::try_from(self.0.len() as u64).unwrap()
            .write(buf).map_err(|_| InsufficientSpace)?;

        // Put the rest of the segment into the buffer
        buf.put_slice(&self.0);

        // Done.
        return Ok(());
    }
}

enum Message {
    Unordered {
        channel: ChannelId,
        payload: Bytes,
    },

    Sequenced {
        channel: ChannelId,
        sequence: Sequence<u16>,
        payload: Bytes,
    },
}

// TODO: Reduce code repetition because there's a lot
impl Message {
    fn read<B: Buf>(buf: &mut B) -> Result<Self, MessageDecodeError> {
        let code: u64 = decode_varint(buf)?.into();

        match code {
            0 => {
                let channel = decode_varint(buf)?.into();
                let length: usize = u64::from(decode_varint(buf)?)
                    .try_into().map_err(|_| MessageDecodeError::TooLarge)?;
                if buf.remaining() < length { return Err(MessageDecodeError::EndOfInput); }
                let payload = buf.copy_to_bytes(length);

                return Ok(Self::Unordered {
                    channel,
                    payload,
                })
            },

            1 => {
                let channel = decode_varint(buf)?.into();
                if buf.remaining() < 2 { return Err(MessageDecodeError::EndOfInput); }
                let sequence = buf.get_u16().into();
                let length: usize = u64::from(decode_varint(buf)?)
                    .try_into().map_err(|_| MessageDecodeError::TooLarge)?;
                if buf.remaining() < length { return Err(MessageDecodeError::EndOfInput); }
                let payload = buf.copy_to_bytes(length);

                return Ok(Self::Sequenced {
                    channel,
                    sequence,
                    payload,
                })
            },

            _ => return Err(MessageDecodeError::UnknownCode),
        }
    }

    fn write<B: BufMut>(&self, buf: &mut B) -> Result<(), InsufficientSpace> {
        match self {
            Message::Unordered {
                channel,
                payload,
            } => {
                let code = VarInt::from_u32(0);

                // Calculate the amount of space needed to write the message
                // If it's too low, we can return an error early
                let channel = VarInt::from(*channel);
                let pld_len = VarInt::try_from(payload.len()).unwrap(); // once again unwrapping is fine because a 46,116 tb message is ridiculous
                let b_len_sum = payload.len() + (code.len() + channel.len() + pld_len.len()) as usize;
                if buf.remaining_mut() < b_len_sum { return Err(InsufficientSpace); }

                // Write everything
                // Unwrapping is fine since we checked that there's enough space
                code.write(buf).unwrap();
                channel.write(buf).unwrap();
                pld_len.write(buf).unwrap();
                buf.put_slice(&payload);
            },

            Message::Sequenced {
                channel,
                sequence,
                payload,
            } => {
                let code = VarInt::from_u32(1);

                // Calculate the amount of space needed to write the message
                // If it's too low, we can return an error early
                let channel = VarInt::from(*channel);
                let seq_len = 2; // amount of bytes a u16 takes up, we don't encode it as a varint
                let pld_len = VarInt::try_from(payload.len()).unwrap(); // once again unwrapping is fine because a 46,116 tb message is ridiculous
                let b_len_sum = payload.len() + seq_len + (code.len() + channel.len() + pld_len.len()) as usize;
                if buf.remaining_mut() < b_len_sum { return Err(InsufficientSpace); }

                // Write everything
                // Unwrapping is fine since we checked that there's enough space
                code.write(buf).unwrap();
                channel.write(buf).unwrap();
                buf.put_u16(sequence.inner());
                pld_len.write(buf).unwrap();
                buf.put_slice(&payload);
            },
        }

        return Ok(());
    }
}

enum MessageDecodeError {
    UnknownCode,
    EndOfInput,
    TooLarge,
}

impl From<EndOfInput> for MessageDecodeError {
    fn from(value: EndOfInput) -> Self {
        Self::EndOfInput
    }
}

struct EndOfInput;
struct InsufficientSpace;

fn decode_varint<B: Buf>(buf: &mut B) -> Result<VarInt, EndOfInput> {
    VarInt::read(buf).map_err(|_| EndOfInput)
}
/// The default writer allocation amount, in bytes.
const DEFAULT_WRITER_ALLOCATION: usize = 256;

/// A trait for bitstream writers, used in serialising network data.
pub trait BitWriter {
    /// Creates a new instance of the BitWriter.
    fn new() -> Self;
    /// Expands the buffer to allow the writing of at least `bytes` more bytes.
    fn allocate_bytes(&mut self, bytes: usize);

    /// Writes a single bit to the buffer.
    fn write_bit(&mut self, data: bool);
    /// Writes a byte (u8) to the buffer.
    fn write_byte(&mut self, data: u8);
    /// Writes four bytes (u32) to the buffer.
    fn write_u32(&mut self, data: u32);
    /// Writes eight bytes (u64) to the buffer.
    fn write_u64(&mut self, data: u64);

    /// Reads an iterator of single bits and writes it to the buffer.
    fn write_bits<T: Iterator<Item = bool>>(&mut self, iter: T);
    /// Reads an iterator of bytes (u8) and writes it to the buffer.
    fn write_bytes<T: Iterator<Item = u8>>(&mut self, iter: T);

    /// Consumes the `BitWriter`, returning a series of bytes for network transport.
    fn to_bytes(self) -> Vec<u8>;
}

/// A trait for bitstream readers, used in deserialising network data.
pub trait BitReader {
    /// Reads a single bit, advancing the buffer.
    fn read_bit(&mut self) -> Result<bool, BitstreamError>;
    /// Reads a byte (u8), advancing the buffer.
    fn read_u8(&mut self) -> Result<u8, BitstreamError>;
    /// Reads four bytes (u32), advancing the buffer.
    fn read_u32(&mut self) -> Result<u32, BitstreamError>;
    /// Reads eight bytes (u64), advancing the buffer.
    fn read_u64(&mut self) -> Result<u64, BitstreamError>;

    /// Returns a simple iterator over bytes in the buffer.
    fn iter(&mut self) -> BitReaderIter where Self: Sized {
        BitReaderIter::new(self)
    }
    /// Reads `amount` bytes from the buffer and returns them.
    fn read_bytes(&mut self, amount: usize) -> Result<Vec<u8>, BitstreamError> where Self: Sized {
        let mut vec = Vec::with_capacity(amount);
        let mut iter = self.iter();
        for _ in 0..amount {
            match iter.next() {
                Some(byte) => vec.push(byte),
                None => return Err(BitstreamError),
            }
        }

        Ok(vec)
    }
}

/// Allows manual bitstream serialisation rather than using Bevy reflection.
pub trait ManualBitSerialisation {
    fn serialise(&self, writer: &mut impl BitWriter);
    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> where Self: Sized;
}

/// A simple iterator over bytes in a [BitReader].
pub struct BitReaderIter<'a> {
    reader: &'a mut dyn BitReader,
}

impl<'a> BitReaderIter<'a> {
    fn new(reader: &'a mut impl BitReader) -> Self {
        Self { reader }
    }
}

impl<'a> Iterator for BitReaderIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.reader.read_u8();
        match byte {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitstreamError;

/// Simple [BitWriter] implementation.
// Based heavily off of this code by Connor Carpenter:
// https://github.com/naia-lib/naia/blob/15bf23adda3d8c36f1464c7fa09e0c16d40d3622/shared/serde/src/bit_writer.rs#L15-L22
// Licensed under the MIT license.
pub struct DefaultBitWriter {
    scratch: u8,
    scratch_index: u8,
    buffer: Vec<u8>,
}

impl BitWriter for DefaultBitWriter {
    fn new() -> Self {
        Self {
            scratch: 0,
            scratch_index: 0,
            buffer: Vec::with_capacity(DEFAULT_WRITER_ALLOCATION),
        }
    }

    fn allocate_bytes(&mut self, bytes: usize) {
        self.buffer.reserve_exact(bytes);
    }

    fn write_bit(&mut self, data: bool) {
        self.scratch <<= 1;
        if data { self.scratch |= 1; }
        self.scratch_index += 1;
        if self.scratch_index >= 8 {
            self.buffer.push(self.scratch.reverse_bits());
            self.scratch_index -= 8;
            self.scratch = 0;
        }
    }

    fn write_byte(&mut self, data: u8) {
        let mut temp = data;
        for _ in 0..8 {
            self.write_bit(temp & 1 != 0);
            temp >>= 1;
        }
    }

    fn write_u32(&mut self, data: u32) {
        let mut temp = data;
        for _ in 0..32 {
            self.write_bit(temp & 1 != 0);
            temp >>= 1;
        }
    }

    fn write_u64(&mut self, data: u64) {
        let mut temp = data;
        for _ in 0..64 {
            self.write_bit(temp & 1 != 0);
            temp >>= 1;
        }
    }

    fn write_bits<T: Iterator<Item = bool>>(&mut self, iter: T) {
        for bit in iter {
            self.write_bit(bit);
        }
    }

    fn write_bytes<T: Iterator<Item = u8>>(&mut self, iter: T) {
        for byte in iter {
            self.write_byte(byte);
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        self.buffer
    }
}
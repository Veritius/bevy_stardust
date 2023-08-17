use super::octetstring::Octet;

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

    /// Writes an octet to the buffer.
    fn write_octet(&mut self, data: Octet);

    /// Writes four octets, as a u32, to the buffer.
    /// 
    /// This is intended for efficient in-memory representation of eight octets, and cannot accurately transport numbers by itself.
    /// Always use the `ser` method to prevent issues related to endian-ness.
    fn write_four(&mut self, data: u32);

    /// Writes eight octets, as a u64, to the buffer.
    /// 
    /// This is intended for efficient in-memory representation of eight octets, and cannot accurately transport numbers by itself.
    /// Always use the `ser` method to prevent issues related to endian-ness.
    fn write_eight(&mut self, data: u64);

    /// Reads an iterator of single bits and writes it to the buffer.
    fn write_bits<T: Iterator<Item = bool>>(&mut self, iter: T);

    /// Reads an iterator of octets and writes it to the buffer.
    fn write_bytes<T: Iterator<Item = Octet>>(&mut self, iter: T);

    /// Consumes the `BitWriter`, returning an array of bytes for network transport.
    fn to_bytes(self) -> Box<[Octet]>;
}

/// A trait for bitstream readers, used in deserialising network data.
pub trait BitReader {
    /// Reads a single bit, advancing the buffer.
    fn read_bit(&mut self) -> Result<bool, BitstreamError>;

    /// Reads an octet, advancing the buffer.
    fn read_octet(&mut self) -> Result<Octet, BitstreamError>;

    /// Reads four octets, as a u32, advancing the buffer.
    /// 
    /// This is intended for efficient in-memory representation of eight octets, and cannot accurately read numbers by itself.
    /// Always use the `de` method to prevent issues related to endian-ness.
    fn read_four(&mut self) -> Result<u32, BitstreamError>;

    /// Reads eight octets, as a u64, advancing the buffer.
    /// 
    /// This is intended for efficient in-memory representation of eight octets, and cannot accurately read numbers by itself.
    /// Always use the `de` method to prevent issues related to endian-ness.
    fn read_eight(&mut self) -> Result<u64, BitstreamError>;

    /// Returns a simple iterator over bytes in the buffer.
    fn iter(&mut self) -> BitReaderIter where Self: Sized {
        BitReaderIter::new(self)
    }

    /// Reads `amount` bytes from the buffer and returns them.
    fn read_bytes(&mut self, amount: usize) -> Result<Vec<Octet>, BitstreamError> where Self: Sized {
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
    type Item = Octet;

    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.reader.read_octet();
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

    fn write_octet(&mut self, data: u8) {
        let mut temp = data;
        for _ in 0..8 {
            self.write_bit(temp & 1 != 0);
            temp >>= 1;
        }
    }

    fn write_four(&mut self, data: u32) {
        let mut temp = data;
        for _ in 0..32 {
            self.write_bit(temp & 1 != 0);
            temp >>= 1;
        }
    }

    fn write_eight(&mut self, data: u64) {
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
            self.write_octet(byte);
        }
    }

    fn to_bytes(self) -> Box<[u8]> {
        self.buffer.into_boxed_slice()
    }
}

/// Simple [BitReader] implementation
// Based heavily off this code by Connor Carpenter:
// https://github.com/naia-lib/naia/blob/15bf23adda3d8c36f1464c7fa09e0c16d40d3622/shared/serde/src/bit_reader.rs
// Licensed under the MIT License.
pub struct DefaultBitReader<'a> {
    buffer: &'a [u8],
    scratch: u8,
    scratch_index: u8,
    buffer_index: usize,
}

impl<'a> DefaultBitReader<'a> {
    pub fn from_slice<'b: 'a>(slice: &'b [u8]) -> Self {
        Self {
            buffer: slice,
            scratch: 0,
            scratch_index: 0,
            buffer_index: 0,
        }
    }
}

impl BitReader for DefaultBitReader<'_> {
    fn read_bit(&mut self) -> Result<bool, BitstreamError> {
        if self.scratch_index == 0 {
            if self.buffer_index == self.buffer.len() {
                return Err(BitstreamError);
            }

            self.scratch = self.buffer[self.buffer_index];
            self.buffer_index += 1;
            self.scratch_index += 8;
        }

        let value = self.scratch & 1;
        self.scratch >>= 1;
        self.scratch_index -= 1;

        Ok(value != 0)
    }

    fn read_octet(&mut self) -> Result<u8, BitstreamError> {
        let mut output = 0;

        for _ in 0..7 {
            if self.read_bit()? {
                output |= 128;
            }
            output >>= 1;
        }
        
        if self.read_bit()? {
            output |= 128;
        }

        Ok(output)
    }

    fn read_four(&mut self) -> Result<u32, BitstreamError> {
        let mut bytes = [0u8; 4];
        for i in 0..3 {
            bytes[i] = self.read_octet()?;
        }
        Ok(u32::from_ne_bytes(bytes))
    }

    fn read_eight(&mut self) -> Result<u64, BitstreamError> {
        let mut bytes = [0u8; 8];
        for i in 0..7 {
            bytes[i] = self.read_octet()?;
        }
        Ok(u64::from_ne_bytes(bytes))
    }
}
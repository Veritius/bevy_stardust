use std::{cmp::Ordering, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign}, time::Instant};
use bytes::{BufMut, Bytes};
use unbytes::{EndOfInput, Reader};
use crate::{sequences::SequenceId, varint::VarInt};

#[derive(Debug, Clone)]
pub(crate) struct RecvFrame {
    pub ftype: FrameType,
    pub order: Option<SequenceId>,
    pub ident: Option<VarInt>,
    pub payload: Bytes,
}

impl RecvFrame {
    pub(super) fn read(reader: &mut Reader) -> Result<Self, FrameReadError> {
        // Get the byte for frame flags
        let flags: FrameFlags = reader.read_u8()
        .map_err(|_| FrameReadError::UnexpectedEnd)?
        .into();

        // Get the frame flags from the bitfield.
        let no_payload = flags.any_high(FrameFlags::NO_PAYLOAD);
        let has_ident = flags.any_high(FrameFlags::IDENTIFIED);
        let has_order = flags.any_high(FrameFlags::ORDERED);

        // Check that the flags are valid
        if no_payload && has_order  { return Err(FrameReadError::IncompatibleFlags); }
        if no_payload && !has_ident { return Err(FrameReadError::IncompatibleFlags); }

        // Parse the frame header type
        let ftype: FrameType = reader
        .read_u8()
        .map_err(|_| FrameReadError::UnexpectedEnd)?
        .try_into()
        .map_err(|_| FrameReadError::UnknownFrameType)?;

        // Get the frame channel id if present
        let ident = match has_ident {
            false => None,
            true => Some(VarInt::read(reader)
                .map_err(|_| FrameReadError::InvalidFrameIdent)?),
        };

        // Get the frame channel ordering if present
        let order = match has_order {
            false => None,
            true => Some(reader.read_u16()
                .map_err(|_| FrameReadError::UnexpectedEnd)?.into()),
        };

        // Return a payload object, or make an empty one if there is no payload
        let payload = if no_payload { Bytes::new() } else {
            // Read the length of the packet
            let len: usize = VarInt::read(reader)
            .map_err(|_| FrameReadError::InvalidFrameLength)?
            .into();

            // Read the next few bytes as per len
            reader.read_bytes(len)
                .map_err(|_| FrameReadError::UnexpectedEnd)?
        };

        // Return the frame
        return Ok(RecvFrame { ftype, order, ident, payload });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameReadError {
    UnexpectedEnd,
    IncompatibleFlags,
    UnknownFrameType,
    InvalidFrameLength,
    InvalidFrameIdent,
}

impl From<EndOfInput> for FrameReadError {
    fn from(_: EndOfInput) -> Self {
        Self::UnexpectedEnd
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SendFrame {
    pub priority: u32,
    pub time: Instant,
    pub ftype: FrameType,
    pub reliable: bool,
    pub order: Option<SequenceId>,
    pub ident: Option<VarInt>,
    pub payload: Bytes,
}

impl SendFrame {
    /// Return an estimate for how many bytes this frame will take up when serialised.
    /// The estimate must be equal to or greater than the real value, or panics will occur.
    pub fn size(&self) -> usize {
        // Always takes up a certain amount of data
        // (flags + type + payload)
        let mut estimate = 2 + self.payload.len();

        // The size of the frame is dependent on if the payload is of length zero
        // Payloads with length zero have various optimisations to reduce their size
        if self.payload.len() != 0 {
            // Ordering id takes always takes up two bytes
            if self.order.is_some() { estimate += 2 }

            // Estimate of the length varint
            // Unwrapping is fine because the payload would have
            // to be a ridiculous size before it errors out.
            // There isn't a computer on the planet (at the time of writing)
            // that can store enough information to trigger a panic here.
            estimate += VarInt::size_u64(self.payload.len() as u64).unwrap();
        }

        // Identifier takes up space as well
        if let Some(ident) = self.ident {
            estimate += ident.size();
        }

        return estimate;
    }

    pub(super) fn write<B: BufMut>(&self, mut b: B) {
        #[cfg(debug_assertions)]
        let (rem, est) = (b.remaining_mut(), self.size());

        let mut flags = FrameFlags::EMPTY;
        if self.ident.is_some() { flags |= FrameFlags::IDENTIFIED; }

        let payload_length = self.payload.len();

        if payload_length > 0 {
            // Flag as ordered only if the payload length is non-zero.
            // This is because zero-length payloads have no reason to be
            // ordered, as they are all functionally identical and do the same thing.
            if self.order.is_some() { flags |= FrameFlags::ORDERED; }
        } else {
            // Assert because a no-payload frame with no
            // identifier is meaningless and is probably a bug.
            debug_assert!(flags.any_high(FrameFlags::IDENTIFIED));

            // Flag as a frame with no payload
            flags |= FrameFlags::NO_PAYLOAD;
        }

        // Put the frame flags and type
        // into the buffer first
        b.put_u8(flags.into());
        b.put_u8(self.ftype.into());

        if let Some(ident) = self.ident {
            ident.write(&mut b);
        }

        // Put in some extra data only if the payload is non-zero.
        // These fields will only appear under this condition.
        if payload_length > 0 {
            // If the frame has an ordering, put it in!
            if let Some(order) = self.order {
                b.put_u16(order.into());
            }

            // Put the payload length into the bin using a varint
            // Unwrapping is fine since I doubt anyone will try to
            // send a payload with a length of 4,611 petabytes.
            // Not that there's any computers that can even store that.
            VarInt::try_from(self.payload.len()).unwrap().write(&mut b);

            // Put in the payload itself
            b.put(&self.payload[..]);
        }

        #[cfg(debug_assertions)] {
            let new_rem = b.remaining_mut();
            debug_assert_eq!(new_rem, rem - est);
        }
    }
}

impl PartialEq for SendFrame {
    fn eq(&self, other: &Self) -> bool {
        if self.priority != other.priority { return false }
        if self.time != other.time { return false }
        return true
    }
}

impl Eq for SendFrame {}

impl PartialOrd for SendFrame {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SendFrame {
    fn cmp(&self, other: &Self) -> Ordering {
        const PRIORITY_MULTIPLIER: i128 = 256;

        let priority_diff = (self.priority as i128 - other.priority as i128) * PRIORITY_MULTIPLIER;
        let instant_diff = match self.time.cmp(&other.time) {
            Ordering::Less => other.time.duration_since(self.time).as_millis() as i128,
            Ordering::Greater => -(self.time.duration_since(other.time).as_millis() as i128),
            Ordering::Equal => 0,
        };

        return (priority_diff + instant_diff).cmp(&0);
    }
}

#[test]
fn frame_ord_test() {
    use std::time::Duration;

    fn frame(priority: u32, time: Instant) -> SendFrame {
        SendFrame {
            priority, time,
            ftype: FrameType::Control,
            reliable: false,
            order: None,
            ident: None,
            payload: Bytes::from_static(&[]),
        }
    }

    let now = Instant::now();

    let a = frame(1, now);
    let b = frame(1, now);
    assert_eq!(a.cmp(&b), Ordering::Equal);
    assert_eq!(b.cmp(&a), Ordering::Equal);

    let a = frame(10, now);
    let b = frame(1, now);
    assert_eq!(a.cmp(&b), Ordering::Greater);
    assert_eq!(b.cmp(&a), Ordering::Less);

    let dur = Duration::from_secs(1);
    let a = frame(1, now - dur);
    let b = frame(1, now);
    assert_eq!(a.cmp(&b), Ordering::Greater);
    assert_eq!(b.cmp(&a), Ordering::Less);

    let dur = Duration::from_secs(1);
    let a = frame(10, now);
    let b = frame(1, now - dur);
    assert_eq!(a.cmp(&b), Ordering::Greater);
    assert_eq!(b.cmp(&a), Ordering::Less);

    let dur = Duration::from_secs(600);
    let a = frame(10, now);
    let b = frame(1, now - dur);
    assert_eq!(a.cmp(&b), Ordering::Less);
    assert_eq!(b.cmp(&a), Ordering::Greater);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameType {
    Control,
    Stardust,
}

impl TryFrom<u8> for FrameType {
    type Error = u8;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Control),
            1 => Ok(Self::Stardust),
            _ => Err(value),
        }
    }
}

impl From<FrameType> for u8 {
    fn from(value: FrameType) -> Self {
        match value {
            FrameType::Control => 0,
            FrameType::Stardust => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct FrameFlags(u8);

impl FrameFlags {
    pub const EMPTY: Self = Self(0);

    pub const NO_PAYLOAD : Self = Self(1 << 0);
    pub const IDENTIFIED : Self = Self(1 << 1);
    pub const ORDERED    : Self = Self(1 << 2);

    #[inline]
    pub fn any_high(&self, mask: FrameFlags) -> bool {
        return (*self & mask).0 > 0;
    }
}

impl From<u8> for FrameFlags {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<FrameFlags> for u8 {
    #[inline]
    fn from(value: FrameFlags) -> Self {
        value.0
    }
}

impl BitOr for FrameFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for FrameFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAnd for FrameFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for FrameFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl std::fmt::Debug for FrameFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FrameFlags").field(&format_args!("{:16b}", self.0)).finish()
    }
}

pub(super) struct FrameQueue {
    queue: Vec<SendFrame>,
}

impl FrameQueue {
    #[inline]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn assess(&self) -> FrameQueueStats {
        let mut stats = FrameQueueStats {
            total_frames_count: self.queue.len(),
            reliable_frames_count: 0,
            unreliable_frames_count: 0,
            total_bytes_estimate: 0,
        };

        self.queue.iter().for_each(|frame| {
            stats.total_bytes_estimate += frame.size();

            match frame.reliable {
                true => stats.reliable_frames_count += 1,
                false => stats.unreliable_frames_count += 1,
            }
        });

        return stats;
    }

    pub fn push(&mut self, frame: SendFrame) {
        // Add to the queue
        self.queue.push(frame);
    }

    pub fn iter<'a>(&'a mut self) -> FrameQueueIter<'a> {
        // Sort packets
        self.queue.sort_unstable();

        // Return iterator
        FrameQueueIter {
            inner: &mut self.queue
        }
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            queue: Vec::with_capacity(size),
        }
    }
}

#[derive(Clone)]
pub(super) struct FrameQueueStats {
    pub total_frames_count: usize,
    pub reliable_frames_count: usize,
    pub unreliable_frames_count: usize,
    pub total_bytes_estimate: usize,
}

pub(crate) struct FrameQueueIter<'a> {
    inner: &'a mut Vec<SendFrame>,
}

impl<'a> FrameQueueIter<'a> {
    pub fn finish(self, iter: impl Iterator<Item = SendFrame>) {
        for frame in iter {
            self.inner.push(frame);
        }
    }
}

impl Iterator for FrameQueueIter<'_> {
    type Item = SendFrame;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop()
    }
}

#[test]
fn frame_queue_test() {
    static PAYLOAD: Bytes = Bytes::from_static(&[]);

    fn dummy(priority: u32, time: Instant) -> SendFrame {
        SendFrame {
            priority, time,
            ftype: FrameType::Control,
            reliable: false,
            order: None,
            ident: None,
            payload: PAYLOAD.clone(),
        }
    }

    let mut queue = FrameQueue::with_capacity(16);
    let time = Instant::now();

    queue.push(dummy(100, time));
    queue.push(dummy(15, time));
    queue.push(dummy(76, time));
    queue.push(dummy(512, time));

    let mut iter = queue.iter();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();
    assert_eq!(a.cmp(&b), Ordering::Greater);

    let c = iter.next().unwrap();
    assert_eq!(b.cmp(&c), Ordering::Greater);

    let d = iter.next().unwrap();
    assert_eq!(c.cmp(&d), Ordering::Greater);

    assert_eq!(iter.next(), None);
}
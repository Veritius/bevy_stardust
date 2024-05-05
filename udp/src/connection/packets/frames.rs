use std::{cmp::Ordering, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign}, time::Instant};
use bytes::Bytes;
use tracing::trace_span;
use crate::{sequences::SequenceId, varint::VarInt};

#[derive(Debug, Clone)]
pub(crate) struct RecvFrame {
    pub flags: FrameFlags,
    pub ftype: FrameType,
    pub order: Option<SequenceId>,
    pub ident: Option<VarInt>,
    pub payload: Bytes,
}

#[derive(Debug, Clone)]
pub(crate) struct SendFrame {
    pub priority: u32,
    pub time: Instant,
    pub flags: FrameFlags,
    pub ftype: FrameType,
    pub reliable: bool,
    pub order: Option<SequenceId>,
    pub ident: Option<VarInt>,
    pub payload: Bytes,
}

impl SendFrame {
    /// Return an estimate for how many bytes this frame will take up when serialised.
    /// The estimate must be equal to or greater than the real value, or panics will occur.
    pub fn bytes_est(&self) -> usize {
        // Always takes up a certain amount of data
        // (flags + type + payload)
        let mut estimate = 2 + self.payload.len();

        // Ordering id takes always takes up two bytes
        if self.order.is_some() { estimate += 2 }

        // Identifier takes up space as well
        if let Some(ident) = self.ident {
            estimate += ident.estimate_size();
        }

        // Estimate is one
        return estimate;
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
            flags: FrameFlags::EMPTY,
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

impl FrameType {
    /// The size of a frame type code, in bytes.
    pub const WIRE_SIZE: usize = 1;
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
pub(crate) struct FrameFlags(u8);

impl FrameFlags {
    pub const EMPTY: Self = Self(0);

    pub const ORDERED    : Self = Self(1 << 0);
    pub const IDENTIFIED : Self = Self(1 << 1);

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
    pub fn assess(&self) -> FrameQueueStats {
        let mut stats = FrameQueueStats {
            total_frames_count: self.queue.len(),
            reliable_frames_count: 0,
            unreliable_frames_count: 0,
            total_bytes_estimate: 0,
        };

        self.queue.iter().for_each(|frame| {
            stats.total_bytes_estimate += frame.bytes_est();

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
        let trace_span = trace_span!("Sorting frames for packing");
        trace_span.in_scope(|| {
            self.queue.sort_unstable();
        });

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
            flags: FrameFlags::EMPTY,
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
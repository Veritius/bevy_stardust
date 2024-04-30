use std::{cmp::Ordering, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign}, time::Instant, vec::Drain};
use bytes::Bytes;
use tracing::trace_span;

#[derive(Debug, Clone)]
pub(crate) struct Frame {
    pub priority: u32,
    pub time: Instant,
    pub flags: FrameFlags,
    pub ftype: FrameType,
    pub payload: Bytes,
}

impl Frame {
    pub fn bytes_est(&self) -> usize {
        // Always takes up at least as many bytes at the header + payload
        let mut estimate = FrameType::WIRE_SIZE + self.payload.len();

        // Ordered frames take up an additional 2 bytes for their sequence id
        if self.flags.any_high(FrameFlags::ORDERED) { estimate += 2;}

        // Estimate is one
        return estimate;
    }
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        if self.priority != other.priority { return false }
        if self.time != other.time { return false }
        return true
    }
}

impl Eq for Frame {}

impl PartialOrd for Frame {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Frame {
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

    fn frame(priority: u32, time: Instant) -> Frame {
        Frame {
            priority, time,
            flags: FrameFlags::EMPTY,
            ftype: FrameType::Control,
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
pub(crate) struct FrameFlags(u16);

impl FrameFlags {
    pub const EMPTY: Self = Self(0);

    pub const RELIABLE: Self = Self(1 << 0);
    pub const ORDERED:  Self = Self(1 << 1);

    #[inline]
    pub fn any_high(&self, mask: FrameFlags) -> bool {
        return (*self & mask).0 > 0;
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
    queue: Vec<Frame>,
    total_byte_est: usize,
    no_rel_byte_est: usize,
    rel_byte_est: usize,
}

impl FrameQueue {
    pub fn push(&mut self, frame: Frame) {
        // Estimate counter which adjusts from flags
        let bytes_estimate = frame.bytes_est();
        self.total_byte_est += bytes_estimate;

        // Individual counters for reliable and unreliable frames
        match frame.flags.any_high(FrameFlags::RELIABLE) {
            true  => { self.rel_byte_est += bytes_estimate    },
            false => { self.no_rel_byte_est += bytes_estimate },
        }

        // Add to the queue
        self.queue.push(frame);
    }

    pub fn iter<'a>(&'a mut self) -> FrameQueueIter<'a> {
        // Reset counters
        self.total_byte_est = 0;
        self.no_rel_byte_est = 0;
        self.rel_byte_est = 0;

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

    #[inline]
    pub fn total_est(&self) -> usize {
        self.total_byte_est
    }

    #[inline]
    pub fn unreliable_est(&self) -> usize {
        self.no_rel_byte_est
    }

    #[inline]
    pub fn reliable_est(&self) -> usize {
        self.rel_byte_est
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            queue: Vec::with_capacity(size),
            total_byte_est: 0,
            no_rel_byte_est: 0,
            rel_byte_est: 0,
        }
    }
}

pub(crate) struct FrameQueueIter<'a> {
    inner: &'a mut Vec<Frame>,
}

impl Iterator for FrameQueueIter<'_> {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop()
    }
}

#[test]
fn frame_queue_test() {
    static PAYLOAD: Bytes = Bytes::from_static(&[]);

    fn dummy(priority: u32, time: Instant) -> Frame {
        Frame {
            priority, time,
            flags: FrameFlags::EMPTY,
            ftype: FrameType::Control,
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
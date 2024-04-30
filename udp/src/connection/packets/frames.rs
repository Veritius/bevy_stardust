use std::{cmp::Ordering, time::Instant};
use bytes::Bytes;

#[derive(Clone)]
pub(crate) struct Frame {
    pub priority: u32,
    pub time: Instant,
    pub ftype: FrameType,
    pub payload: Bytes,
}

impl Frame {
    pub fn bytes_est(&self) -> usize {
        todo!()
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

#[derive(Clone, Copy, PartialEq, Eq)]
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

pub(super) struct FrameQueue {
    queue: Vec<Frame>,
    byte_est: usize,
}

impl FrameQueue {
    pub fn push(&mut self, frame: Frame) {
        self.byte_est += frame.bytes_est();
        self.queue.push(frame);
    }

    pub fn drain<'a>(&'a mut self) -> impl Iterator<Item = Frame> + 'a {
        self.queue.sort_unstable();
        self.queue.drain(..)
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            queue: Vec::with_capacity(size),
            byte_est: 0,
        }
    }
}
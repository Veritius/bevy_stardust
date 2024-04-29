use std::{cmp::Ordering, time::Instant};
use bevy_stardust::channels::ChannelId;
use bytes::Bytes;

pub(in crate::connection) struct Frame {
    pub priority: u32,
    pub instant: Instant,
    pub inner: FrameInner,
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        if self.priority != other.priority { return false }
        if self.instant != other.instant { return false }
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
        let instant_diff = match self.instant.cmp(&other.instant) {
            Ordering::Less => other.instant.duration_since(self.instant).as_millis() as i128,
            Ordering::Greater => -(self.instant.duration_since(other.instant).as_millis() as i128),
            Ordering::Equal => 0,
        };

        return (priority_diff + instant_diff).cmp(&0);
    }
}

pub(in crate::connection) enum FrameInner {
    Control(ControlFrame),
    Handshake(HandshakeFrame),
    Stardust(StardustFrame),
}

pub(in crate::connection) struct ControlFrame {
    pub payload: Bytes,
}

pub(in crate::connection) struct HandshakeFrame {
    pub payload: Bytes,
}

pub(in crate::connection) struct StardustFrame {
    pub channel: ChannelId,
    pub payload: Bytes,
}

#[test]
fn frame_ord_test() {
    use std::time::Duration;

    fn dummy() -> FrameInner {
        FrameInner::Control(ControlFrame { payload: Bytes::from_static(&[]) })
    }

    let now = Instant::now();

    let a = Frame { priority: 1, instant: now, inner: dummy() };
    let b = Frame { priority: 1, instant: now, inner: dummy() };
    assert_eq!(a.cmp(&b), Ordering::Equal);
    assert_eq!(b.cmp(&a), Ordering::Equal);

    let a = Frame { priority: 10, instant: now, inner: dummy() };
    let b = Frame { priority: 1, instant: now, inner: dummy() };
    assert_eq!(a.cmp(&b), Ordering::Greater);
    assert_eq!(b.cmp(&a), Ordering::Less);

    let dur = Duration::from_secs(1);
    let a = Frame { priority: 1, instant: now - dur, inner: dummy() };
    let b = Frame { priority: 1, instant: now, inner: dummy() };
    assert_eq!(a.cmp(&b), Ordering::Greater);
    assert_eq!(b.cmp(&a), Ordering::Less);

    let dur = Duration::from_secs(1);
    let a = Frame { priority: 10, instant: now, inner: dummy() };
    let b = Frame { priority: 1, instant: now - dur, inner: dummy() };
    assert_eq!(a.cmp(&b), Ordering::Greater);
    assert_eq!(b.cmp(&a), Ordering::Less);

    let dur = Duration::from_secs(600);
    let a = Frame { priority: 10, instant: now, inner: dummy() };
    let b = Frame { priority: 1, instant: now - dur, inner: dummy() };
    assert_eq!(a.cmp(&b), Ordering::Less);
    assert_eq!(b.cmp(&a), Ordering::Greater);
}
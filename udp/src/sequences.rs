use std::{cmp::Ordering, fmt::{Debug, Display}, ops::{Add, AddAssign, Sub, SubAssign}};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct SequenceId(pub u16);

impl SequenceId {
    pub const MIN: Self = Self(u16::MIN);
    pub const MID: u16 = 32768;
    pub const MAX: Self = Self(u16::MAX);

    #[inline]
    pub fn new(val: u16) -> Self {
        Self::from(val)
    }

    #[inline]
    pub fn random() -> Self {
        Self(fastrand::u16(..))
    }

    pub fn wrapping_diff(&self, other: &Self) -> u16 {
        let a = self.0;
        let b = other.0;

        let diff = a.abs_diff(b);
        if diff < Self::MID { return diff }

        if a > b {
            return b.wrapping_sub(a);
        } else {
            return a.wrapping_sub(b);
        }
    }
}

impl Default for SequenceId {
    #[inline]
    fn default() -> Self {
        Self::MIN
    }
}

impl PartialOrd for SequenceId {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SequenceId {
    fn cmp(&self, other: &Self) -> Ordering {
        // An adaptation of Glenn Fiedler's wrapping sequence identifier algorithm, modified to output an Ordering
        // https://www.gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/
        if self == other { return Ordering::Equal }
        let a = self.0; let b = other.0;
        let r = ((a>b)&&(a-b<=Self::MID))||((a<b)&&(b-a>Self::MID));
        match r {
            true => Ordering::Greater,
            false => Ordering::Less,
        }
    }
}

impl Add for SequenceId {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_add(rhs.0))
    }
}

impl Add<u16> for SequenceId {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0.wrapping_add(rhs))
    }
}

impl AddAssign for SequenceId {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_add(rhs.0)
    }
}

impl AddAssign<u16> for SequenceId {
    #[inline]
    fn add_assign(&mut self, rhs: u16) {
        self.0 = self.0.wrapping_add(rhs)
    }
}

impl Sub for SequenceId {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<u16> for SequenceId {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u16) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign for SequenceId {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0)
    }
}

impl SubAssign<u16> for SequenceId {
    #[inline]
    fn sub_assign(&mut self, rhs: u16) {
        self.0 = self.0.wrapping_sub(rhs)
    }
}

impl From<u16> for SequenceId {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<SequenceId> for u16 {
    #[inline]
    fn from(value: SequenceId) -> Self {
        value.0
    }
}

impl From<SequenceId> for usize {
    #[inline]
    fn from(value: SequenceId) -> Self {
        value.0 as usize
    }
}

impl Debug for SequenceId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for SequenceId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[test]
fn sequence_id_difference_test() {
    const MIDPOINT: SequenceId = SequenceId(SequenceId::MID);

    #[inline]
    fn seq(v: u16) -> SequenceId {
        SequenceId::from(v)
    }

    assert_eq!(seq(0).wrapping_diff(&seq(0)), 0);
    assert_eq!(seq(0).wrapping_diff(&seq(1)), 1);
    assert_eq!(seq(3).wrapping_diff(&seq(7)), 4);
    assert_eq!(seq(1).wrapping_diff(&seq(0)), 1);
    assert_eq!(seq(7).wrapping_diff(&seq(3)), 4);
    assert_eq!(seq(u16::MAX).wrapping_diff(&seq(u16::MIN)), 1);
    assert_eq!(seq(u16::MAX).sub(3).wrapping_diff(&seq(u16::MIN).add(3)), 7);
    assert_eq!(seq(u16::MIN).wrapping_diff(&seq(u16::MAX)), 1);
    assert_eq!(seq(u16::MIN).add(3).wrapping_diff(&seq(u16::MAX).sub(3)), 7);
    assert_eq!(MIDPOINT.wrapping_diff(&MIDPOINT), 0);
    assert_eq!(MIDPOINT.sub(1).wrapping_diff(&MIDPOINT), 1);
    assert_eq!(MIDPOINT.add(1).wrapping_diff(&MIDPOINT), 1);
}

#[test]
fn sequence_id_ordering_test() {
    const MIDPOINT: SequenceId = SequenceId(SequenceId::MID);

    #[inline]
    fn seq(v: u16) -> SequenceId {
        SequenceId::from(v)
    }

    assert_eq!(seq(4).cmp(&seq(4)), Ordering::Equal);
    assert_eq!(seq(15).cmp(&seq(9)), Ordering::Greater);
    assert_eq!(seq(9).cmp(&seq(15)), Ordering::Less);
    assert_eq!(seq(65534).cmp(&seq(66)), Ordering::Less);
    assert_eq!(seq(u16::MAX).cmp(&seq(u16::MIN)), Ordering::Less);
    assert_eq!(seq(66).cmp(&seq(65534)), Ordering::Greater);
    assert_eq!(seq(u16::MIN).cmp(&seq(u16::MAX)), Ordering::Greater);
    assert_eq!(MIDPOINT.cmp(&MIDPOINT), Ordering::Equal);
    assert_eq!(MIDPOINT.sub(1).cmp(&MIDPOINT), Ordering::Less);
    assert_eq!(MIDPOINT.add(1).cmp(&MIDPOINT), Ordering::Greater);
}
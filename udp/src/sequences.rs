use std::{cmp::Ordering, fmt::{Debug, Display}, ops::{Add, AddAssign, Sub, SubAssign}};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SequenceId(pub u16);

impl SequenceId {
    pub const MIDPOINT: u16 = 32768;

    pub fn diff(&self, other: &Self) -> u16 {
        let a = self.0;
        let b = other.0;

        let diff = a.abs_diff(b);
        if diff > Self::MIDPOINT { return diff }

        if a > b {
            return b.wrapping_sub(a);
        } else {
            return a.wrapping_sub(b);
        }
    }

    #[inline]
    pub fn random() -> Self {
        Self(fastrand::u16(..))
    }
}

impl PartialOrd for SequenceId {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SequenceId {
    // An adaptation of Glenn Fiedler's wrapping sequence identifier algorithm
    // https://www.gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/
    fn cmp(&self, other: &Self) -> Ordering {
        match self.diff(other).cmp(&Self::MIDPOINT) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Less => self.0.cmp(&other.0),
            Ordering::Greater => {
                match self.0.cmp(&other.0) {
                    Ordering::Less => Ordering::Greater,
                    Ordering::Greater => Ordering::Less,
                    Ordering::Equal => unreachable!(),
                }
            },
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
fn sequence_id_ordering_test() {
    const MIDPOINT: SequenceId = SequenceId(SequenceId::MIDPOINT);

    #[inline]
    fn seq(v: u16) -> SequenceId {
        SequenceId::from(v)
    }

    assert_eq!(seq(4).cmp(&seq(4)), Ordering::Equal);
    assert_eq!(seq(15).cmp(&seq(9)), Ordering::Less);
    assert_eq!(seq(9).cmp(&seq(15)), Ordering::Greater);
    assert_eq!(seq(65534).cmp(&seq(66)), Ordering::Less);
    assert_eq!(seq(66).cmp(&seq(65534)), Ordering::Greater);
    assert_eq!(MIDPOINT.sub(1).cmp(&MIDPOINT), Ordering::Greater);
    assert_eq!(MIDPOINT.add(1).cmp(&MIDPOINT), Ordering::Less);
    assert_eq!(MIDPOINT.cmp(&MIDPOINT), Ordering::Equal);
}
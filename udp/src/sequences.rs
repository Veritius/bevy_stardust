use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SequenceId(pub u16);

impl SequenceId {
    pub const MIDPOINT: u16 = u16::MAX / 2;

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
}

impl PartialOrd for SequenceId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SequenceId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        todo!()
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

// Glenn Fiedler's wrap-around sequence identifier algorithm
// https://www.gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/
pub fn sequence_greater_than(s1: u16, s2: u16) -> bool {
    ((s1 > s2) && (s1 - s2 <= 32768)) || ((s1 < s2) && (s2 - s1 > 32768))
}

/// Returns the minimum difference between the absolute difference and a wrapping difference.
#[deprecated]
pub fn wrapping_diff(a: u16, b: u16) -> u16 {
    const MIDPOINT: u16 = u16::MAX / 2;

    let diff = a.abs_diff(b);
    match (a > b, diff > MIDPOINT) {
        (_, false) => diff,
        (true, _) => b.wrapping_sub(a),
        (false, _) => a.wrapping_sub(b),
    }
}

#[test]
fn test_wrapping_diff() {
    assert_eq!(wrapping_diff(0, 1), 1);
    assert_eq!(wrapping_diff(1, 3), 2);
    assert_eq!(wrapping_diff(15, 35), 20);
    assert_eq!(wrapping_diff(u16::MAX, u16::MIN), 1);
    assert_eq!(wrapping_diff(u16::MAX-1, u16::MIN+1), 3);
}
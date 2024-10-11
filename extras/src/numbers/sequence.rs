use std::{cmp::Ordering, ops::{Add, AddAssign, Sub, SubAssign}};

/// A sequence value that always wraps.
/// 
/// When you are sending a sequence of items, you may want to identify them with a unique number.
/// However, if you reach the limit of representable values for a type like `u32`, you cannot send further items.
/// This is what `Sequence<T>` solves. Mutation always wraps around, and comparison takes wrapping into account.
/// However, this type is only suitable for values that **only increment** and will only increment a certain
/// amount in a certain span of time. If you can receive more than 1/3 the range of values of your `Sequence<T>`
/// at once, you should use a `T` that can represent more values.
/// 
/// The `Ord` implementation takes into account the wrapping difference between the two values.
/// A **very high** sequence number is considered **lesser** than a **very low** sequence number.
/// Since we know the value wraps, we can assume that, for a `Sequence<u8>`, `0` was sent *after* `255`,
/// since we only [`increment`](Self::increment) the sequence value a certain amount. For example, the
/// difference between `4` and `9` is `5`, but the difference between `254` and `1` is `3`, again
/// assuming you're using a `Sequence<u8>`.
/// 
/// `T` can be any one of `u8`, `u16`, `u32`, `u64`, or `u128`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Sequence<T: Sequential>(T);

impl<T: Sequential> From<T> for Sequence<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Sequential> Sequence<T> {
    /// Returns the inner integer value.
    #[inline]
    pub fn inner(&self) -> T {
        self.0
    }

    /// Increment the value by `1`. Wraps at numerical bounds.
    pub fn increment(&mut self) {
        self.0.increment();
    }

    /// Returns the difference between two sequence values.
    pub fn diff(&self, other: &Self) -> T {
        let a = self.0;
        let b = other.0;

        let diff = a.abs_diff(b);
        if diff < T::MID { return diff }

        if a > b {
            return b.wrapping_sub(a);
        } else {
            return a.wrapping_sub(b);
        }
    }
}

impl<T: Sequential> PartialOrd for Sequence<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Sequential> Ord for Sequence<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // An adaptation of Glenn Fiedler's wrapping sequence identifier algorithm, modified to output an Ordering
        // https://www.gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/
        if self == other { return Ordering::Equal }
        let a = self.0; let b = other.0;
        let r = ((a>b)&&(a-b<=T::MID))||((a<b)&&(b-a>T::MID));
        match r {
            true => Ordering::Greater,
            false => Ordering::Less,
        }
    }
}

impl<T: Sequential> PartialEq<T> for Sequence<T> {
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}

impl<T: Sequential> Add<T> for Sequence<T> {
    type Output = Sequence<T>;

    #[inline]
    fn add(self, rhs: T) -> Self::Output {
        Sequence(self.0.wrapping_add(rhs))
    }
}

impl<T: Sequential> AddAssign<T> for Sequence<T> {
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs;
    }
}

impl<T: Sequential> Sub<T> for Sequence<T> {
    type Output = Sequence<T>;

    #[inline]
    fn sub(self, rhs: T) -> Self::Output {
        Sequence(self.0.wrapping_sub(rhs))
    }
}

impl<T: Sequential> SubAssign<T> for Sequence<T> {
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - rhs;
    }
}

/// A number that can be used in a [`Sequence`] value.
pub trait Sequential
where
    Self: sealed::Sealed,
    Self: Sized + Clone + Copy + Default + Ord,
    Self: Add<Output = Self> + Sub<Output = Self>,
{
    /// The minimum representable value.
    const MIN: Self;

    /// The value between `MIN` and `MAX`.
    const MID: Self;

    /// The maximum representable value.
    const MAX: Self;

    /// Increment the value by `1`.
    fn increment(&mut self);

    /// Absolute difference between two values.
    /// 
    /// `abs_diff(50, 120)` is the same as `abs_diff(-50, -120)`.
    fn abs_diff(self, other: Self) -> Self;

    /// Subtraction that wraps at numerical bounds.
    fn wrapping_sub(self, other: Self) -> Self;

    /// Addition that wraps at numerical bounds.
    fn wrapping_add(self, other: Self) -> Self;
}

macro_rules! impl_seqvalue {
    ($type:ty, $val:expr) => {
        impl Sequential for $type {
            const MIN: $type = <$type>::MIN;
            const MID: $type = $val;
            const MAX: $type = <$type>::MAX;

            #[inline]
            fn increment(&mut self) {
                *self = *self + 1;
            }

            #[inline]
            fn abs_diff(self, other: Self) -> Self {
                self.abs_diff(other)
            }

            #[inline]
            fn wrapping_sub(self, other: Self) -> Self {
                self.wrapping_sub(other)
            }

            #[inline]
            fn wrapping_add(self, other: Self) -> Self {
                self.wrapping_add(other)
            }
        }
    };
}

impl_seqvalue!(u8, 2u8.pow(7));
impl_seqvalue!(u16, 2u16.pow(15));
impl_seqvalue!(u32, 2u32.pow(31));
impl_seqvalue!(u64, 2u64.pow(63));
impl_seqvalue!(u128, 2u128.pow(127));

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for u128 {}
}

#[test]
fn sequence_id_difference_test() {
    const MIDPOINT: Sequence::<u16> = Sequence(u16::MID);

    #[inline]
    fn seq(v: u16) -> Sequence<u16> {
        Sequence::from(v)
    }

    assert_eq!(seq(0).diff(&seq(0)), 0);
    assert_eq!(seq(0).diff(&seq(1)), 1);
    assert_eq!(seq(3).diff(&seq(7)), 4);
    assert_eq!(seq(1).diff(&seq(0)), 1);
    assert_eq!(seq(7).diff(&seq(3)), 4);
    assert_eq!(seq(u16::MAX).diff(&seq(u16::MIN)), 1);
    assert_eq!(seq(u16::MAX).sub(3).diff(&seq(u16::MIN).add(3)), 7);
    assert_eq!(seq(u16::MIN).diff(&seq(u16::MAX)), 1);
    assert_eq!(seq(u16::MIN).add(3).diff(&seq(u16::MAX).sub(3)), 7);
    assert_eq!(MIDPOINT.diff(&MIDPOINT), 0);
    assert_eq!(MIDPOINT.sub(1).diff(&MIDPOINT), 1);
    assert_eq!(MIDPOINT.add(1).diff(&MIDPOINT), 1);
}

#[test]
fn sequence_id_ordering_test() {
    const MIDPOINT: Sequence::<u16> = Sequence(u16::MID);

    #[inline]
    fn seq(v: u16) -> Sequence<u16> {
        Sequence::from(v)
    }

    assert_eq!(seq(4).partial_cmp(&seq(4)), Some(Ordering::Equal));
    assert_eq!(seq(15).partial_cmp(&seq(9)), Some(Ordering::Greater));
    assert_eq!(seq(9).partial_cmp(&seq(15)), Some(Ordering::Less));
    assert_eq!(seq(65534).partial_cmp(&seq(66)), Some(Ordering::Less));
    assert_eq!(seq(u16::MAX).partial_cmp(&seq(u16::MIN)), Some(Ordering::Less));
    assert_eq!(seq(66).partial_cmp(&seq(65534)), Some(Ordering::Greater));
    assert_eq!(seq(u16::MIN).partial_cmp(&seq(u16::MAX)), Some(Ordering::Greater));
    assert_eq!(MIDPOINT.partial_cmp(&MIDPOINT), Some(Ordering::Equal));
    assert_eq!(MIDPOINT.sub(1).partial_cmp(&MIDPOINT), Some(Ordering::Less));
    assert_eq!(MIDPOINT.add(1).partial_cmp(&MIDPOINT), Some(Ordering::Greater));

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

#[cfg(feature="octs")]
mod octs {
    use octs::{Encode, FixedEncodeLen, Decode};
    use super::{Sequence, Sequential};

    impl<T: Sequential + Encode> Encode for Sequence<T> {
        type Error = T::Error;

        #[inline]
        fn encode(&self, mut dst: impl octs::Write) -> Result<(), octs::BufTooShortOr<Self::Error>> {
            self.0.encode(&mut dst)
        }
    }

    impl<T: Sequential + FixedEncodeLen> FixedEncodeLen for Sequence<T> {
        const ENCODE_LEN: usize = T::ENCODE_LEN;
    }

    impl<T: Sequential + Decode> Decode for Sequence<T> {
        type Error = T::Error;

        #[inline]
        fn decode(mut src: impl octs::Read) -> Result<Self, octs::BufTooShortOr<Self::Error>> {
            T::decode(&mut src).map(|v| Self(v))
        }
    }
}
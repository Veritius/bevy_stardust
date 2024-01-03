use std::time::Instant;
use bytes::Bytes;

mod river;

struct SentPacket {
    pub data: Bytes,
    pub time: Instant,
}

// Glenn Fiedler's wrap-around sequence identifier algorithm
// https://www.gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/
fn sequence_greater_than(s1: u16, s2: u16) -> bool {
    ((s1 > s2) && (s1 - s2 <= 32768)) || ((s1 < s2) && (s2 - s1 > 32768))
}

/// Returns the minimum difference between the absolute difference and a wrapping difference.
fn wrapping_diff(a: u16, b: u16) -> u16 {
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
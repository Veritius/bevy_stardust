mod pipe;

// Glenn Fiedler's wrap-around sequence identifier algorithm
// https://www.gafferongames.com/post/reliability_ordering_and_congestion_avoidance_over_udp/
fn sequence_greater_than(s1: u16, s2: u16) -> bool {
    ((s1 > s2) && (s1 - s2 <= 32768)) || ((s1 < s2) && (s2 - s1 > 32768))
}
pub(super) fn best_fit(
    bins: impl Iterator<Item = (usize, usize)>,
    length: usize,
) -> usize {
    // Pick the most suitable bin
    let mut most_suitable = (usize::MAX, usize::MAX);
    for (index, bin) in bins.enumerate() {
        let remaining_space = bin.0.saturating_sub(bin.1);
        if remaining_space < length { continue }
        if remaining_space > most_suitable.1 { continue }
        most_suitable = (index, remaining_space)
    }

    return most_suitable.0
}
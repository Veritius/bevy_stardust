pub(super) fn best_fit(
    bins: &Vec<Vec<u8>>,
    length: usize,
) -> usize {
    // Pick the most suitable bin
    let mut most_suitable = (usize::MAX, usize::MAX);
    for (index, bin) in bins.iter().enumerate() {
        let remaining_space = bin.capacity().saturating_sub(bin.len());
        if remaining_space < length { continue }
        if remaining_space > most_suitable.1 { continue }
        most_suitable = (index, remaining_space)
    }

    return most_suitable.0
}
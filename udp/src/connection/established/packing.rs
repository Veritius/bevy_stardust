use std::ops::Deref;

pub(super) struct PackingManager {
    bin_size: usize,
    bins: Vec<Vec<u8>>,
}

impl PackingManager {
    /// Returns a new `PackingManager` that uses bins of size `capacity`.
    pub fn new(capacity: usize) -> Self {
        Self {
            bin_size: capacity,
            bins: vec![],
        }
    }

    /// Reallocates all bins to have the same capacity.
    pub fn resize(&mut self, size: usize) {
        self.bin_size = size;
        for bin in &mut self.bins {
            bin.shrink_to(self.bin_size);
        }
    }

    /// Packs some bytes into a bin according to a [`PackingAlgorithm`].
    pub fn push<Algorithm: PackingAlgorithm>(&mut self, bytes: &[u8]) {
        // Check the length of the slice isn't longer than our bin size
        // TODO: Maybe handle this without panicking
        assert!(self.bin_size >= bytes.len(), "Can't pack a slice larger than the bins");

        // If the bin storage is empty, create a new bin
        if self.bins.is_empty() {
            let mut bin = Vec::with_capacity(self.bin_size);
            bin.extend_from_slice(bytes);
            return;
        }

        // Run the packing algorithm
        let index = Algorithm::pack(bytes.len(), &mut self.bins
            .iter()
            .enumerate()
            .map(|(index, v)| { (index, v.capacity() - v.len()) }));

        // Create a new bin if value is max
        if index == usize::MAX {
            let mut bin = Vec::with_capacity(self.bin_size);
            bin.extend_from_slice(bytes);
            return;
        }

        // Index the bin of choice and append
        self.bins[index].extend_from_slice(bytes);
    }

    /// Pop the fullest bin, if any. After use, the bin is emptied.
    /// `filter` takes the current size of the bin, returning true if acceptable, used to filter out bins that are too small.
    pub fn pop(&mut self, filter: impl Fn(usize) -> bool) -> Option<BinRef> {
        // Finds the maximum value
        let result = self.bins
            .iter_mut()
            .enumerate()
            .filter(|(_,v)| filter(v.len()))
            .max_by(|(_,a),(_,b)| a.len().cmp(&b.len()));

        // If an acceptable bin was found, return it
        // Otherwise, return none
        if let Some((index, bin)) = result {
            // Effectively pops from an index without shuffling the vec around
            // Since the BinRef returns the bin to its rightful place, this is fine
            let mut g_bin = Vec::with_capacity(0); // doesn't allocate
            std::mem::swap(bin, &mut g_bin);

            // Create the binref object
            let binref = BinRef {
                manager: self,
                bin_idx: index,
                bin: g_bin,
            };

            // Return it
            return Some(binref);
        } else {
            // No suitable bin
            return None;
        }
    }
}

/// Dereferences to a byte slice, which is the contents of the bin.
/// When this is dropped, the bin is returned to the manager and cleared.
pub(super) struct BinRef<'a> {
    manager: &'a mut PackingManager,
    bin_idx: usize,
    bin: Vec<u8>,
}

impl Deref for BinRef<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bin
    }
}

impl Drop for BinRef<'_> {
    fn drop(&mut self) {
        // Clear the bin
        self.bin.clear();

        // Return the bin allocation to the manager
        std::mem::swap(&mut self.manager.bins[self.bin_idx], &mut self.bin)
    }
}

/// An online algorithm for bin packing.
pub(super) trait PackingAlgorithm {
    /// From an iterator of potential bins (index, available space) return the index of the most suitable bin.
    /// If the returned index is equal to `usize::MAX` a new bin is created instead.
    fn pack(item: usize, bins: &mut impl Iterator<Item = (usize, usize)>) -> usize;
}

pub(super) struct BestFit;

impl PackingAlgorithm for BestFit {
    fn pack(item: usize, bins: &mut impl Iterator<Item = (usize, usize)>) -> usize {
        bins
        .filter(|(_, rem)| *rem >= item)
        .min_by(|(_, rem_a), (_, rem_b)| rem_a.cmp(rem_b))
        .map(|(index, _)| index)
        .unwrap_or(usize::MAX)
    }
}

pub(super) struct FirstFit;

impl PackingAlgorithm for FirstFit {
    fn pack(item: usize, bins: &mut impl Iterator<Item = (usize, usize)>) -> usize {
        bins
        .find(|(_, rem)| *rem >= item)
        .map(|(index, _)| index)
        .unwrap_or(usize::MAX)
    }
}
use std::collections::BTreeMap;

pub(crate) struct ReliabilityData {
    local: u16,
    remote: u16,
    bitfield: u32,
    unacked: BTreeMap<u16, Box<[u8]>>,
}

impl ReliabilityData {
    pub fn increment_local(&mut self) -> u16 {
        let local = self.local;
        self.local += 1;
        local
    }

    pub fn increment_remote(&mut self, remote: u16) {
        if !sequence_greater_than(remote, self.remote) { return }

        todo!()
    }

    pub fn bitfield(&self) -> u32 {
        self.bitfield
    }
}

#[inline]
pub(super) const fn sequence_greater_than(s1: u16, s2: u16) -> bool {
    ((s1>s2)&&(s1-s2<=32768))||((s1<s2)&&(s2-s1>32768))
}
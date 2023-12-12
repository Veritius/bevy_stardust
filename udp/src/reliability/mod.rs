mod pipes;

use self::pipes::Pipes;

pub(crate) struct ReliabilityData {
    pipes: Pipes,
}

impl ReliabilityData {
    pub fn new(pipes: u8) -> Self {
        Self {
            pipes: Pipes::new(pipes),
        }
    }
}

#[inline]
const fn sequence_greater_than(s1: u16, s2: u16) -> bool {
    ((s1>s2)&&(s1-s2<=32768))||((s1<s2)&&(s2-s1>32768))
}
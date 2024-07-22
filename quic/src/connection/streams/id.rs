#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamId(u64);

impl StreamId {
    pub const MAX: u64 = 2u64.pow(62) - 1;

    pub fn new(inner: u64) -> Result<Self, ()> {
        if inner > Self::MAX { return Err(()); }
        return Ok(Self(inner));
    }

    pub fn inner(self) -> u64 {
        self.0
    }
}
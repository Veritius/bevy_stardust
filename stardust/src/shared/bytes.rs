//! Traits for converting between byte storage types.

pub trait OwnedByteStore {
    fn into_boxed(self) -> Box<[u8]>;
    fn into_vec(self) -> Vec<u8>;
}

impl OwnedByteStore for Vec<u8> {
    fn into_boxed(self) -> Box<[u8]> {
        self.into_boxed_slice()
    }

    fn into_vec(self) -> Vec<u8> {
        self
    }
}

impl OwnedByteStore for Box<[u8]> {
    fn into_boxed(self) -> Box<[u8]> {
        self
    }

    fn into_vec(self) -> Vec<u8> {
        self.into_vec()
    }
}
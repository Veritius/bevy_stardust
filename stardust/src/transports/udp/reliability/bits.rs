mod private {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
}

pub trait SequenceNumber: private::Sealed {

}

impl SequenceNumber for u8 {

}

impl SequenceNumber for u16 {

}

impl SequenceNumber for u32 {

}

pub trait SequenceBitset: private::Sealed {

}

impl SequenceBitset for u8 {

}

impl SequenceBitset for u16 {

}

impl SequenceBitset for u32 {

}